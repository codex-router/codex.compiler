/* test2.c */
#include <stdio.h>

typedef struct { int x; int y; } Point;
typedef enum { RED, GREEN, BLUE } Color;

int fibonacci(int n) {
    if (n <= 1) return n;
    return fibonacci(n - 1) + fibonacci(n - 2);
}

void swap(int *a, int *b) {
    int tmp = *a;
    *a = *b;
    *b = tmp;
}

int main(void) {
    int arr[5];
    int i;
    for (i = 0; i < 5; i++) {
        arr[i] = i;
    }
    printf("Sum = %d\n", arr[0]);

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
    swap(&y, &arr[0]);

    do {
        y--;
    } while (y > 0);

    return 0;
}
