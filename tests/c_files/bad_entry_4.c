#include <stdint.h>
#include <stdlib.h>


void utx0(void);
void utx1(void *addr, size_t len);


static uint64_t shared_array[100];


void bad_entry_4()
{
	utx1(&shared_array, sizeof (shared_array));
	shared_array[102] += 1;
}

int main()
{
	bad_entry_4();
	return 0;
}
