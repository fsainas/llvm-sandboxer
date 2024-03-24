#include <stdint.h>
#include <stdlib.h>


void utx0(void);
void utx1(void *addr, size_t len);


static uint64_t shared_array[100];


void good_entry_3()
{
	utx1(&shared_array, sizeof (shared_array));
	shared_array[1] += 1;
}

int main()
{
	good_entry_3();
	return 0;
}
