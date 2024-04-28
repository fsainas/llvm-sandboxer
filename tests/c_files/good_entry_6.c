#include <stdint.h>
#include <stdlib.h>


void utx0(void);
void utx1(void *addr, size_t len);


static uint64_t shared_array[100];


void good_entry_6()
{
	utx1(&shared_array, sizeof (shared_array));
	int i = 3+1;
	shared_array[i] += 1;
}

int main()
{
	good_entry_6();
	return 0;
}
