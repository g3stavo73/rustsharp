#include <stdio.h>

long rustsharp_add(long a, long b);
long rustsharp_fibonacci(unsigned int n);

int main() {
    printf("add: %ld\n", rustsharp_add(10, 20));
    printf("fib: %ld\n", rustsharp_fibonacci(10));
    return 0;
}
