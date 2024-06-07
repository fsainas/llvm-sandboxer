#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <time.h>

#define ARR_LENGTH 10000000

void utx0(void);
void utx1(void *addr, size_t len);

static uint64_t shared_array[ARR_LENGTH];

void simple_store()
{
    srand(time(NULL));

    for (int i = 0; i < ARR_LENGTH; i++) {
        //int index = rand() % ARR_LENGTH;
        shared_array[i] = rand();
    }

}

int main()
{

    simple_store();
	return 0;
}
