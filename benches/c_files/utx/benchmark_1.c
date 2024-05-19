#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <time.h>

#define ARR_LENGTH 10000

void utx0(void);
void utx1(void *addr, size_t len);

static uint64_t shared_array[ARR_LENGTH];

void benchmark_1() {

	utx1(&shared_array, sizeof (shared_array));

    for(int i = 0; i < ARR_LENGTH; i++) {

        for(int j = 0; j < ARR_LENGTH - 1; j++) {

            if(shared_array[j] > shared_array[j+1]) {
                int tmp = shared_array[j];
                shared_array[j] = shared_array[j+1];
                shared_array[j+1] = tmp;
            }

        }

    }

}

int main()
{
    srand(time(NULL));

    for (int i = 0; i < ARR_LENGTH; i++) {
        shared_array[i] = rand() % ARR_LENGTH;
        //printf("%lu ", shared_array[i]);
    }

    benchmark_1();

    for (int i = 0; i < ARR_LENGTH; i++) {
        //printf("%lu ", shared_array[i]);
    }
	return 0;
}
