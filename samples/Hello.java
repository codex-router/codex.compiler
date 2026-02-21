// samples/Hello.java – valid Java
package samples;

import java.util.ArrayList;
import java.util.List;
import java.util.Map;
import java.util.HashMap;
import java.util.stream.Collectors;

public class Hello {

    // Constants
    public static final int MAX_SIZE = 100;
    private static final String GREETING = "Hello, World!";

    // Generics example
    public static <T extends Comparable<T>> T findMax(List<T> list) {
        if (list == null || list.isEmpty()) {
            throw new IllegalArgumentException("List must not be empty");
        }
        T max = list.get(0);
        for (T item : list) {
            if (item.compareTo(max) > 0) {
                max = item;
            }
        }
        return max;
    }

    // Interface
    interface Transformer<A, B> {
        B transform(A input);
        default String describe() { return "Transformer"; }
    }

    // Enum with body
    enum Direction {
        NORTH(0, 1), SOUTH(0, -1), EAST(1, 0), WEST(-1, 0);

        private final int dx, dy;

        Direction(int dx, int dy) {
            this.dx = dx;
            this.dy = dy;
        }

        public int[] delta() { return new int[]{ dx, dy }; }
    }

    // Abstract class
    static abstract class Shape {
        protected String color;

        public Shape(String color) {
            this.color = color;
        }

        public abstract double area();

        @Override
        public String toString() {
            return getClass().getSimpleName() + "[color=" + color + ", area=" + area() + "]";
        }
    }

    static class Circle extends Shape {
        private double radius;

        public Circle(String color, double radius) {
            super(color);
            this.radius = radius;
        }

        @Override
        public double area() {
            return Math.PI * radius * radius;
        }
    }

    static class Rectangle extends Shape {
        private double w, h;

        public Rectangle(String color, double w, double h) {
            super(color);
            this.w = w;
            this.h = h;
        }

        @Override
        public double area() { return w * h; }
    }

    // Nested generic class
    static class Pair<A, B> {
        private A first;
        private B second;

        public Pair(A first, B second) {
            this.first = first;
            this.second = second;
        }

        public A getFirst() { return first; }
        public B getSecond() { return second; }
    }

    public static void main(String[] args) {
        System.out.println(GREETING);

        // Collections
        List<Integer> numbers = new ArrayList<>();
        for (int i = 0; i < MAX_SIZE; i++) {
            numbers.add(i * i);
        }
        System.out.println("Max: " + findMax(numbers));

        // Enhanced for, try-catch
        int sum = 0;
        try {
            for (int n : numbers) {
                sum += n;
            }
        } catch (Exception e) {
            System.err.println("Error: " + e.getMessage());
        } finally {
            System.out.println("Sum computed: " + sum);
        }

        // Switch expression (Java 14+)
        int day = 3;
        String dayName;
        switch (day) {
            case 1: dayName = "Monday"; break;
            case 2: dayName = "Tuesday"; break;
            case 3: dayName = "Wednesday"; break;
            default: dayName = "Other"; break;
        }
        System.out.println("Day: " + dayName);

        // Shapes
        List<Shape> shapes = new ArrayList<>();
        shapes.add(new Circle("red", 5.0));
        shapes.add(new Rectangle("blue", 3.0, 4.0));
        for (Shape s : shapes) {
            System.out.println(s);
        }

        // Map
        Map<String, Integer> freq = new HashMap<>();
        for (String arg : args) {
            freq.merge(arg, 1, Integer::sum);
        }

        // Lambda / method reference (Transformer interface)
        Transformer<String, Integer> lengthOf = s -> s.length();
        System.out.println("Length: " + lengthOf.transform("hello"));

        // Ternary & instanceof
        Object obj = "test";
        String result = (obj instanceof String) ? (String) obj : "not a string";
        System.out.println("Result: " + result);

        // Assert
        assert sum >= 0 : "sum must be non-negative";

        // Multi-dimensional array
        int[][] matrix = new int[3][3];
        for (int i = 0; i < 3; i++) {
            for (int j = 0; j < 3; j++) {
                matrix[i][j] = i * 3 + j;
            }
        }

        Pair<String, Integer> pair = new Pair<>("answer", 42);
        System.out.println(pair.getFirst() + " = " + pair.getSecond());
    }
}
