#include <stdint.h>
#include <stdlib.h>


void utx0(void);
void utx1(void *addr, size_t len);


static uint64_t shared_array[100];


void bad_entry_1(uint64_t index)
{
	utx0();
	shared_array[index] += 1;
}

int main()
{
	bad_entry_1(1);
	return 0;
}
