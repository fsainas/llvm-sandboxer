#include <stdint.h>
#include <stdlib.h>


void utx0(void);
void utx1(void *addr, size_t len);


static uint64_t shared_array[100];


void bad_entry_5(uint64_t index)
{
	utx1(&shared_array[index], sizeof(&shared_array[index]));
	if (index < 100) {
		shared_array[index+1] += 1; // Accessing the next element
	}
}

int main()
{
	bad_entry_5(7);
	return 0;
}
