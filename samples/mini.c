/* minimal test */
int add(int a, int b) {
    return a + b;
}

int main(void) {
    int x = add(1, 2);
    do {
        x--;
    } while (x > 0);
    return 0;
}
