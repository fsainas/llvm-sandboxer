#include <stdint.h>
#include <stdlib.h>


void utx0(void);
void utx1(void *addr, size_t len);


static uint64_t shared_array[100];


void good_entry_4(uint64_t index)
{
	utx1(&shared_array, sizeof (shared_array));
	if (index < 100) {
		shared_array[index] += 1;
	}
}