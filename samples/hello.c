/* samples/hello.c – valid C */
#include <stdio.h>
#include <stdlib.h>

#define MAX_SIZE 100

typedef struct {
    int x;
    int y;
} Point;

typedef enum { RED, GREEN, BLUE } Color;

static int add(int a, int b) {
    return a + b;
}

void swap(int *a, int *b) {
    int tmp = *a;
    *a = *b;
    *b = tmp;
}

int fibonacci(int n) {
    if (n <= 1) return n;
    return fibonacci(n - 1) + fibonacci(n - 2);
}

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

    Point p = {3, 4};
    printf("Point (%d, %d)\n", p.x, p.y);

    Color c = GREEN;
    switch (c) {
        case RED:   printf("Red\n");   break;
        case GREEN: printf("Green\n"); break;
        case BLUE:  printf("Blue\n");  break;
        default:    printf("?\n");     break;
    }

    unsigned int x = 0xDEADBEEFu;
    int y = (int)x;
    swap(&y, &sum);

    do {
        sum--;
    } while (sum > 0);

    return 0;
}
