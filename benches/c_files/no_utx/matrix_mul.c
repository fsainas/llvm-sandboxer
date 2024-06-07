#include <math.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <time.h>

#define SIZE 250

void utx0(void);
void utx1(void *addr, size_t len);

static uint64_t shared_matrix0[SIZE][SIZE];
static uint64_t shared_matrix1[SIZE][SIZE];
static uint64_t shared_matrix2[SIZE][SIZE];

void matrix_mul()
{

    for (int i = 0; i < SIZE; i++) {
        for (int j = 0; j < SIZE; j++) {
            shared_matrix2[i][j] = 0;
            for (int k = 0; k < SIZE; k++) {
                shared_matrix2[i][j] += shared_matrix0[i][k] * shared_matrix1[k][j];
            }
        }
    }
}

int main()
{
    srand(time(NULL));

    for (int i = 0; i < SIZE; i++) {
        for (int j = 0; j < SIZE; j++) {
            shared_matrix0[i][j] = rand() % SIZE;
            shared_matrix1[i][j] = rand() % SIZE;
        }
    }

    matrix_mul();

    for (int i = 0; i < SIZE; i++) {
        for (int j = 0; j < SIZE; j++) {
            printf("%lu ", shared_matrix2[i][j]);
        }
    }
	return 0;
}
