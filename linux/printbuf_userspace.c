
#include <stdio.h>
#include <linux/printbuf.h>

void prt_printf(struct printbuf *out, const char *fmt, ...)
{
	va_list args;
	int len;

	do {
		va_start(args, fmt);
		len = vsnprintf(out->buf + out->pos, printbuf_remaining(out), fmt, args);
		va_end(args);
	} while (len + 1 >= printbuf_remaining(out) &&
		 !printbuf_make_room(out, len + 1));

	len = min_t(size_t, len,
		  printbuf_remaining(out) ? printbuf_remaining(out) - 1 : 0);
	out->pos += len;
}
