#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <time.h>

#define ARR_LENGTH 10000000

void utx0(void);
void utx1(void *addr, size_t len);


static uint64_t shared_array[ARR_LENGTH];


uint64_t benchmark_0()
{

	utx1(&shared_array, sizeof (shared_array));

    srand(time(NULL));

    // Init array
    for (int i = 0; i < ARR_LENGTH; i++) {
        shared_array[i] = rand() % 100;
    }

    // Compute the sum
    uint64_t sum = 0;
    for (int i = 0; i < ARR_LENGTH; i++) {
        sum += shared_array[i];
    }

    return sum;
	
}

int main()
{
    uint64_t sum = benchmark_0();
    printf("Sum: %lu\n", sum);
	return 0;
}
