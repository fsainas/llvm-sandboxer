#include <stdint.h>

void utx0(void);

void simple_add() 
{
	uint64_t a, b, c;
	a = 1;
	b = 2;
	c = a + b;
	utx0();
}
