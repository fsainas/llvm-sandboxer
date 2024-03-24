#include <stdint.h>
#include <stdlib.h>


void utx0(void);
void utx1(void *addr, size_t len);


static uint64_t shared_array[100];


// This could be a bad entry if index >= 100
void good_entry_1(uint64_t index)
{
	utx1(&shared_array[0], sizeof (shared_array));
	shared_array[index] += 1;
}

int main()
{
	good_entry_1(5);
	return 0;
}
