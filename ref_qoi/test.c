#define QOI_IMPLEMENTATION
#include "qoi.h"
#include <stdio.h>

void decode_to_file(char* in, char* out)
{
	qoi_desc desc = {.width = 0, .height = 0, .channels = 4, .colorspace = 0};

	unsigned char* pixels = (unsigned char*) qoi_read(in, &desc, 4);

	printf("Channels: %d\nColorspace: %d\nDimensions: %d x %d\n", desc.channels, desc.colorspace, desc.width, desc.height);

	FILE *write_ptr;
	write_ptr = fopen(out, "wb");
	fwrite(pixels, 1, desc.width * desc.height * desc.channels, write_ptr);
	fclose(write_ptr);
}

void encode_to_file(char* filename) {
	unsigned char pixels[] = {
		192, 0, 0,
		192, 0, 0,
		0, 0, 192,
		0, 0, 192,
		0, 0, 0,
		0, 0, 0,
		0, 0, 0,
		255, 255, 255,
		128, 128, 128,
		120, 130, 130,
		100, 128, 128,
		125, 125, 125,
		128, 128, 128,
		128, 128, 128,
		255, 255, 255,
		124, 134, 71,
		124, 134, 71,
		124, 134, 71,
		128, 128, 128,
		150, 130, 130,
		150, 130, 130,
		0, 0, 0,
		0, 0, 0,
		0, 0, 0,
	};

	qoi_desc desc = {.width = 6, .height = 4, .channels = 3, .colorspace = 1};

	int out_len = 0;
	qoi_write(filename, pixels, &desc);
}

int main(void)
{
	// encode_to_file("./foo.qoi");
	decode_to_file("../imgs/testcard.qoi", "../imgs/testcard.bin");

	return 0;
}
