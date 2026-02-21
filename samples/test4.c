/* test4.c */
#include <stdio.h>
#define MAX_SIZE 100

int main(void) {
    int arr[MAX_SIZE];
    int i;
    for (i = 0; i < MAX_SIZE; i++) {
        arr[i] = i * 2;
    }
    int sum = 0;
    for (i = 0; i < MAX_SIZE; i++) {
        sum += arr[i];
    }
    printf("Sum = %d\n", sum);
    return 0;
}
