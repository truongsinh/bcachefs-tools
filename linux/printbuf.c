// SPDX-License-Identifier: LGPL-2.1+
/* Copyright (C) 2022 Kent Overstreet */

#include <linux/err.h>
#include <linux/math64.h>
#include <linux/printbuf.h>
#include <linux/slab.h>

#ifdef __KERNEL__
#include <linux/export.h>
#include <linux/kernel.h>
#else
#ifndef EXPORT_SYMBOL
#define EXPORT_SYMBOL(x)
#endif
#endif

static inline size_t printbuf_linelen(struct printbuf *buf)
{
	return buf->pos - buf->last_newline;
}

int printbuf_make_room(struct printbuf *out, unsigned extra)
{
	unsigned new_size;
	char *buf;

	if (!out->heap_allocated)
		return 0;

	/* Reserved space for terminating nul: */
	extra += 1;

	if (out->pos + extra < out->size)
		return 0;

	new_size = roundup_pow_of_two(out->size + extra);
	buf = krealloc(out->buf, new_size, !out->atomic ? GFP_KERNEL : GFP_NOWAIT);

	if (!buf) {
		out->allocation_failure = true;
		return -ENOMEM;
	}

	out->buf	= buf;
	out->size	= new_size;
	return 0;
}
EXPORT_SYMBOL(printbuf_make_room);

/**
 * printbuf_str - returns printbuf's buf as a C string, guaranteed to be null
 * terminated
 */
const char *printbuf_str(const struct printbuf *buf)
{
	/*
	 * If we've written to a printbuf then it's guaranteed to be a null
	 * terminated string - but if we haven't, then we might not have
	 * allocated a buffer at all:
	 */
	return buf->pos
		? buf->buf
		: "";
}
EXPORT_SYMBOL(printbuf_str);

/**
 * printbuf_exit - exit a printbuf, freeing memory it owns and poisoning it
 * against accidental use.
 */
void printbuf_exit(struct printbuf *buf)
{
	if (buf->heap_allocated) {
		kfree(buf->buf);
		buf->buf = ERR_PTR(-EINTR); /* poison value */
	}
}
EXPORT_SYMBOL(printbuf_exit);

void prt_newline(struct printbuf *buf)
{
	unsigned i;

	printbuf_make_room(buf, 1 + buf->indent);

	__prt_char(buf, '\n');

	buf->last_newline	= buf->pos;

	for (i = 0; i < buf->indent; i++)
		__prt_char(buf, ' ');

	printbuf_nul_terminate(buf);

	buf->last_field		= buf->pos;
	buf->tabstop = 0;
}
EXPORT_SYMBOL(prt_newline);

/**
 * printbuf_indent_add - add to the current indent level
 *
 * @buf: printbuf to control
 * @spaces: number of spaces to add to the current indent level
 *
 * Subsequent lines, and the current line if the output position is at the start
 * of the current line, will be indented by @spaces more spaces.
 */
void printbuf_indent_add(struct printbuf *buf, unsigned spaces)
{
	if (WARN_ON_ONCE(buf->indent + spaces < buf->indent))
		spaces = 0;

	buf->indent += spaces;
	while (spaces--)
		prt_char(buf, ' ');
}
EXPORT_SYMBOL(printbuf_indent_add);

/**
 * printbuf_indent_sub - subtract from the current indent level
 *
 * @buf: printbuf to control
 * @spaces: number of spaces to subtract from the current indent level
 *
 * Subsequent lines, and the current line if the output position is at the start
 * of the current line, will be indented by @spaces less spaces.
 */
void printbuf_indent_sub(struct printbuf *buf, unsigned spaces)
{
	if (WARN_ON_ONCE(spaces > buf->indent))
		spaces = buf->indent;

	if (buf->last_newline + buf->indent == buf->pos) {
		buf->pos -= spaces;
		printbuf_nul_terminate(buf);
	}
	buf->indent -= spaces;
}
EXPORT_SYMBOL(printbuf_indent_sub);

/**
 * prt_tab - Advance printbuf to the next tabstop
 *
 * @buf: printbuf to control
 *
 * Advance output to the next tabstop by printing spaces.
 */
void prt_tab(struct printbuf *out)
{
	int spaces = max_t(int, 0, out->tabstops[out->tabstop] - printbuf_linelen(out));

	BUG_ON(out->tabstop > ARRAY_SIZE(out->tabstops));

	prt_chars(out, ' ', spaces);

	out->last_field = out->pos;
	out->tabstop++;
}
EXPORT_SYMBOL(prt_tab);

/**
 * prt_tab_rjust - Advance printbuf to the next tabstop, right justifying
 * previous output
 *
 * @buf: printbuf to control
 *
 * Advance output to the next tabstop by inserting spaces immediately after the
 * previous tabstop, right justifying previously outputted text.
 */
void prt_tab_rjust(struct printbuf *buf)
{
	BUG_ON(buf->tabstop > ARRAY_SIZE(buf->tabstops));

	if (printbuf_linelen(buf) < buf->tabstops[buf->tabstop]) {
		unsigned move = buf->pos - buf->last_field;
		unsigned shift = buf->tabstops[buf->tabstop] -
			printbuf_linelen(buf);

		printbuf_make_room(buf, shift);

		if (buf->last_field + shift < buf->size)
			memmove(buf->buf + buf->last_field + shift,
				buf->buf + buf->last_field,
				min(move, buf->size - 1 - buf->last_field - shift));

		if (buf->last_field < buf->size)
			memset(buf->buf + buf->last_field, ' ',
			       min(shift, buf->size - buf->last_field));

		buf->pos += shift;
		printbuf_nul_terminate(buf);
	}

	buf->last_field = buf->pos;
	buf->tabstop++;
}
EXPORT_SYMBOL(prt_tab_rjust);

enum string_size_units {
	STRING_UNITS_10,	/* use powers of 10^3 (standard SI) */
	STRING_UNITS_2,		/* use binary powers of 2^10 */
};
static int string_get_size(u64 size, u64 blk_size,
			   const enum string_size_units units,
			   char *buf, int len)
{
	static const char *const units_10[] = {
		"B", "kB", "MB", "GB", "TB", "PB", "EB", "ZB", "YB"
	};
	static const char *const units_2[] = {
		"B", "KiB", "MiB", "GiB", "TiB", "PiB", "EiB", "ZiB", "YiB"
	};
	static const char *const *const units_str[] = {
		[STRING_UNITS_10] = units_10,
		[STRING_UNITS_2] = units_2,
	};
	static const unsigned int divisor[] = {
		[STRING_UNITS_10] = 1000,
		[STRING_UNITS_2] = 1024,
	};
	static const unsigned int rounding[] = { 500, 50, 5 };
	int i = 0, j;
	u32 remainder = 0, sf_cap;
	char tmp[13];
	const char *unit;

	tmp[0] = '\0';

	if (blk_size == 0)
		size = 0;
	if (size == 0)
		goto out;

	/* This is Napier's algorithm.  Reduce the original block size to
	 *
	 * coefficient * divisor[units]^i
	 *
	 * we do the reduction so both coefficients are just under 32 bits so
	 * that multiplying them together won't overflow 64 bits and we keep
	 * as much precision as possible in the numbers.
	 *
	 * Note: it's safe to throw away the remainders here because all the
	 * precision is in the coefficients.
	 */
	while (blk_size >> 32) {
		do_div(blk_size, divisor[units]);
		i++;
	}

	while (size >> 32) {
		do_div(size, divisor[units]);
		i++;
	}

	/* now perform the actual multiplication keeping i as the sum of the
	 * two logarithms */
	size *= blk_size;

	/* and logarithmically reduce it until it's just under the divisor */
	while (size >= divisor[units]) {
		remainder = do_div(size, divisor[units]);
		i++;
	}

	/* work out in j how many digits of precision we need from the
	 * remainder */
	sf_cap = size;
	for (j = 0; sf_cap*10 < 1000; j++)
		sf_cap *= 10;

	if (units == STRING_UNITS_2) {
		/* express the remainder as a decimal.  It's currently the
		 * numerator of a fraction whose denominator is
		 * divisor[units], which is 1 << 10 for STRING_UNITS_2 */
		remainder *= 1000;
		remainder >>= 10;
	}

	/* add a 5 to the digit below what will be printed to ensure
	 * an arithmetical round up and carry it through to size */
	remainder += rounding[j];
	if (remainder >= 1000) {
		remainder -= 1000;
		size += 1;
	}

	if (j) {
		snprintf(tmp, sizeof(tmp), ".%03u", remainder);
		tmp[j+1] = '\0';
	}

 out:
	if (i >= ARRAY_SIZE(units_2))
		unit = "UNK";
	else
		unit = units_str[units][i];

	return snprintf(buf, len, "%u%s %s", (u32)size, tmp, unit);
}

/**
 * prt_human_readable_u64 - Print out a u64 in human readable units
 *
 * Units of 2^10 (default) or 10^3 are controlled via @buf->si_units
 */
void prt_human_readable_u64(struct printbuf *buf, u64 v)
{
	printbuf_make_room(buf, 10);
	buf->pos += string_get_size(v, 1, !buf->si_units,
				    buf->buf + buf->pos,
				    printbuf_remaining_size(buf));
}
EXPORT_SYMBOL(prt_human_readable_u64);

/**
 * prt_human_readable_s64 - Print out a s64 in human readable units
 *
 * Units of 2^10 (default) or 10^3 are controlled via @buf->si_units
 */
void prt_human_readable_s64(struct printbuf *buf, s64 v)
{
	if (v < 0)
		prt_char(buf, '-');
	prt_human_readable_u64(buf, abs(v));
}
EXPORT_SYMBOL(prt_human_readable_s64);

/**
 * prt_units_u64 - Print out a u64 according to printbuf unit options
 *
 * Units are either raw (default), or human reabable units (controlled via
 * @buf->human_readable_units)
 */
void prt_units_u64(struct printbuf *out, u64 v)
{
	if (out->human_readable_units)
		prt_human_readable_u64(out, v);
	else
		prt_printf(out, "%llu", v);
}
EXPORT_SYMBOL(prt_units_u64);

/**
 * prt_units_s64 - Print out a s64 according to printbuf unit options
 *
 * Units are either raw (default), or human reabable units (controlled via
 * @buf->human_readable_units)
 */
void prt_units_s64(struct printbuf *out, s64 v)
{
	if (v < 0)
		prt_char(out, '-');
	prt_units_u64(out, abs(v));
}
EXPORT_SYMBOL(prt_units_s64);
