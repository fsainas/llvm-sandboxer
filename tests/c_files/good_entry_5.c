#include <stdint.h>
#include <stdlib.h>


void utx0(void);
void utx1(void *addr, size_t len);


static uint64_t shared_array[100];


void good_entry_5(uint64_t index)
{
	utx1(&shared_array, sizeof (shared_array));
	if (index > 10) {
		shared_array[1] += 1;
	} else if (index < 40) {
		while (shared_array[2] < 100) {
			shared_array[2] += 1;
		}
	} else {
		shared_array[3] += 1;
	}

}

int main()
{
	good_entry_5(5);
	return 0;
}
