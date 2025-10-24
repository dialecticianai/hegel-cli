def factorial(n: int) -> int:
    """Compute n! iteratively and raise ValueError for negative inputs."""
    if n < 0:
        raise ValueError("factorial() not defined for negative values")

    result = 1
    for value in range(2, n + 1):
        result *= value
    return result


if __name__ == "__main__":
    import sys

    if len(sys.argv) != 2:
        print("Usage: python factorial.py <non-negative integer>")
        raise SystemExit(1)

    try:
        number = int(sys.argv[1])
    except ValueError:
        print("Input must be an integer.")
        raise SystemExit(1)

    print(factorial(number))
