#include <stdint.h>
#include <stdlib.h>


void utx0(void);
void utx1(void *addr, size_t len);


static uint64_t shared_array[100];


void bad_entry_3(uint64_t index)
{
	// The size parameter is wrong
	utx1(&shared_array[index], 1);
	shared_array[index] += 1;
}
