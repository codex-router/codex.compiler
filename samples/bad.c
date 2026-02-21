/* samples/bad.c – intentional grammar errors */
#include <stdio.h>

int missing_semicolon(void) {
    int x = 5
    return x;
}

void unclosed_brace(void) {
    if (1) {
        int y = 10;
    /* missing closing brace for function */
}

int main(void) {
    missing_semicolon();
    return 0;
}
