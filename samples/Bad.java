// samples/Bad.java – intentional grammar errors
package samples;

public class Bad {

    // Missing return type
    public doSomething() {
        System.out.println("bad");
    }

    // Missing closing brace for method
    public void anotherMethod() {
        int x = 10;
        if (x > 5) {
            System.out.println("big")
        /* missing semicolon above, missing } for method */
    }

    public static void main(String[] args) {
        new Bad().doSomething();
    }
}
