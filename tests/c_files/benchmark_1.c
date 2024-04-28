#include <stdint.h>
#include <stdlib.h>
#include <stdbool.h>
#include <stdio.h>

#define SIZE 100000000

void utx0(void);
void utx1(void *addr, size_t len);


static uint64_t shared_array[SIZE];

// Function to perform subset sum problem using dynamic programming
bool benchmark_1(uint64_t set[], int n, uint64_t target) {
	utx1(&shared_array[0], sizeof (shared_array));
	bool subset[n + 1][target + 1];
	for (int i = 0; i <= n; i++) {
		subset[i][0] = true;
	}
	for (int i = 1; i <= target; i++) {
		subset[0][i] = false;
	}
	for (int i = 1; i <= n; i++) {
		for (int j = 1; j <= target; j++) {
			if (j < set[i - 1]) {
				subset[i][j] = subset[i - 1][j];
			} else {
				subset[i][j] = subset[i - 1][j] || subset[i - 1][j - set[i - 1]];
			}
		}
	}
	return subset[n][target];
}

int main()
{
	// Initialize the shared array
	for (int i = 0; i < SIZE; i++) {
		shared_array[i] = i + 1; // Example initialization
	}

	// Example target sum
	uint64_t target = 500500; // This is the sum of numbers from 1 to 1000

	// Check if there exists a subset whose sum equals the target sum
	bool result = benchmark_1(shared_array, SIZE, target);

	return 0;
}
