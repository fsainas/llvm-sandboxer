#include <stdint.h>
#include <stdlib.h>
#include <stdio.h>


void utx0(void);
void utx1(void *addr, size_t len);


static uint64_t shared_array[100];


void bad_entry_0(uint64_t index)
{
	shared_array[index] += 1;
}

int main()
{
	bad_entry_0(1);
	return 0;
}
