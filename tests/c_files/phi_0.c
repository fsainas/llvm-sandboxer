#include <stdint.h>
#include <stdlib.h>
#include <math.h>
#include <time.h>

#define SIZE 1000000

void utx0(void);
void utx1(void *addr, size_t len);


static uint64_t shared_array[SIZE];

// This could be a bad entry if index >= 100
void phi_0(uint64_t index)
{
	utx1(&shared_array[0], sizeof (shared_array));
	for (int i = 0; i < SIZE; i++) {
		shared_array[i] = (uint64_t)i*i*i;
	}
}

int main()
{
	phi_0(5);
	return 0;
}
