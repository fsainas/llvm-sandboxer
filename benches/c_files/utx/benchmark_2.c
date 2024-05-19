#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <time.h>

#define ARR_LENGTH 10000000

void utx0(void);
void utx1(void *addr, size_t len);

static uint64_t shared_array[ARR_LENGTH];

void benchmark_2()
{
    srand(time(NULL));

	utx1(&shared_array, sizeof (shared_array));

    for (int i = 0; i < ARR_LENGTH; i++) {
        int index = rand() % ARR_LENGTH;
        shared_array[index] = rand();
    }

}

int main()
{

    benchmark_2();
	return 0;
}
