// samples/hello.cpp – valid C++
#include <iostream>
#include <vector>
#include <string>
#include <algorithm>
#include <memory>

namespace geometry {

    template<typename T>
    class Vector2D {
    public:
        T x, y;

        Vector2D(T x, T y) : x(x), y(y) {}

        Vector2D operator+(const Vector2D& other) const {
            return Vector2D(x + other.x, y + other.y);
        }

        Vector2D& operator+=(const Vector2D& other) {
            x += other.x;
            y += other.y;
            return *this;
        }

        T dot(const Vector2D& other) const {
            return x * other.x + y * other.y;
        }

        virtual ~Vector2D() = default;
    };

    template<typename T>
    class Vector3D : public Vector2D<T> {
    public:
        T z;

        Vector3D(T x, T y, T z) : Vector2D<T>(x, y), z(z) {}

        Vector3D cross(const Vector3D& o) const {
            return { this->y * o.z - z * o.y,
                     z * o.x - this->x * o.z,
                     this->x * o.y - this->y * o.x };
        }
    };

} // namespace geometry

class Shape {
protected:
    std::string name;
public:
    explicit Shape(const std::string& n) : name(n) {}
    virtual double area() const = 0;
    virtual ~Shape() {}
    const std::string& getName() const { return name; }
};

class Circle : public Shape {
    double radius;
public:
    Circle(double r) : Shape("Circle"), radius(r) {}
    double area() const override { return 3.14159 * radius * radius; }
};

class Rectangle : public Shape {
    double w, h;
public:
    Rectangle(double w, double h) : Shape("Rectangle"), w(w), h(h) {}
    double area() const override { return w * h; }
};

template<typename Container>
double total_area(const Container& shapes) {
    double sum = 0.0;
    for (const auto& s : shapes) {
        sum += s->area();
    }
    return sum;
}

int main() {
    std::vector<std::unique_ptr<Shape>> shapes;
    shapes.push_back(std::make_unique<Circle>(5.0));
    shapes.push_back(std::make_unique<Rectangle>(3.0, 4.0));
    shapes.push_back(std::make_unique<Circle>(2.5));

    std::cout << "Total area: " << total_area(shapes) << "\n";

    // Lambda
    auto is_large = [](const std::unique_ptr<Shape>& s) {
        return s->area() > 30.0;
    };
    long large_count = std::count_if(shapes.begin(), shapes.end(), is_large);
    std::cout << "Large shapes: " << large_count << "\n";

    // Range-based for
    for (const auto& shape : shapes) {
        std::cout << shape->getName() << ": " << shape->area() << "\n";
    }

    // Structured bindings (C++17) – skip, use old style
    geometry::Vector2D<int> v1(1, 2), v2(3, 4);
    auto v3 = v1 + v2;
    std::cout << "v3 = (" << v3.x << ", " << v3.y << ")\n";

    try {
        throw std::runtime_error("test exception");
    } catch (const std::exception& e) {
        std::cerr << "Caught: " << e.what() << "\n";
    } catch (...) {
        std::cerr << "Unknown exception\n";
    }

    return 0;
}
