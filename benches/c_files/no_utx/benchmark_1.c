#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <time.h>

#define ARR_LENGTH 10000

void utx0(void);
void utx1(void *addr, size_t len);

static uint64_t shared_array[ARR_LENGTH];

/*
// Function to swap two elements
void swap(uint64_t* a, uint64_t* b) {
    int t = *a;
    *a = *b;
    *b = t;
}

// Partition function
int benchmark_1(uint64_t arr[], uint64_t low, uint64_t high) {
    int pivot = arr[low];  // Choose the last element as pivot
    int i = low;
    int j = high;

    while (i < j) {        

        // condition 1: find the first element greater than 
        // the pivot (from starting) 
        while (arr[i] <= pivot && i <= high - 1) { 
            i++; 
        } 

        // condition 2: find the first element smaller than 
        // the pivot (from last) 
        while (arr[j] > pivot && j >= low + 1) { 
            j--; 
        } 

        if (i < j) { 
            uint64_t t = arr[i];
            arr[i] = arr[j];
            arr[j] = t;
        } 

    }
    uint64_t t = arr[low];
    arr[low] = arr[j];
    arr[j] = t;
    return j;
}

// Quicksort function
void quicksort(uint64_t arr[], uint64_t low, uint64_t high) {
    if (low < high) {
        // pi is partitioning index, arr[pi] is now at the right place
        int pi = benchmark_1(arr, low, high);

        // Recursively sort elements before partition and after partition
        quicksort(arr, low, pi - 1);
        quicksort(arr, pi + 1, high);
    }
}
*/

void benchmark_1() {

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
