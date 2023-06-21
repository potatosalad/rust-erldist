#include <inttypes.h>
#include <stdint.h>
#include <stdio.h>
#include <string.h>

#define MAX_ATOM_CHARACTERS 255
#define MAX_ATOM_SZ_FROM_LATIN1 (2*MAX_ATOM_CHARACTERS)
#define MAX_ATOM_SZ_LIMIT (4*MAX_ATOM_CHARACTERS) /* theoretical byte limit */

typedef int Sint;
typedef unsigned int Uint;
typedef uint8_t byte;
typedef Uint UWord;
typedef Sint SWord;
typedef UWord HashValue;

static void latin1_to_utf8(byte* conv_buf, Uint buf_sz,
                           const byte** srcp, Uint* lenp)
{
    byte* dst;
    const byte* src = *srcp;
    Uint i, len = *lenp;

    // ASSERT(len <= MAX_ATOM_CHARACTERS);
    // ASSERT(buf_sz >= MAX_ATOM_SZ_FROM_LATIN1);

    for (i=0 ; i < len; ++i) {
        if (src[i] & 0x80) {
            goto need_convertion;
        }
    }
    return;

need_convertion:
    memcpy(conv_buf, src, i);
    dst = conv_buf + i;
    for ( ; i < len; ++i) {
        unsigned char chr = src[i];
        if (!(chr & 0x80)) {
            *dst++ = chr;
        }
        else {
            *dst++ = 0xC0 | (chr >> 6);
            *dst++ = 0x80 | (chr & 0x3F);
        }
    }
    *srcp = conv_buf;       
    *lenp = dst - conv_buf;
}

static HashValue
atom_hash(const byte* aname, Uint alen)
{
    const byte* p = aname;
    int len = alen;
    HashValue h = 0, g;
    byte v;

    while(len--) {
        v = *p++;
        /* latin1 clutch for r16 */
        if (len && (v & 0xFE) == 0xC2 && (*p & 0xC0) == 0x80) {
            v = (v << 6) | (*p & 0x3F);
            p++; len--;
        }
        /* normal hashpjw follows for v */
        h = (h << 4) + v;
        if ((g = h & 0xf0000000)) {
            h ^= (g >> 24);
            h ^= g;
        }
    }
    return h;
}

int main(void)
{
    byte utf8_copy[MAX_ATOM_SZ_FROM_LATIN1];
    Uint tlen = 2;
    byte name[2] = {206, 169};
    const byte *text = name;
    Sint no_latin1_chars = tlen;

    memset(utf8_copy, 0, MAX_ATOM_SZ_FROM_LATIN1);

    latin1_to_utf8(utf8_copy, sizeof(utf8_copy), &text, &tlen);
    printf("tlen = %d\nno_latin1_chars = %d\n", tlen, no_latin1_chars);
    for (Sint i = 0; i < tlen; i++) {
        printf("[% 2d] %02x % 3d\n", i, text[i], text[i]);
    }

    printf("hash = %u\n", atom_hash(text, tlen));
    return 0;
}
