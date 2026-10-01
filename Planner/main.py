import sys


def main() -> None:
    in_venv = sys.prefix != sys.base_prefix
    print("Hello from your Python environment!")
    print(f"Python Version: {sys.version.split()[0]}")
    print(f"Executable:     {sys.executable}")
    print(f"Virtual Env:    {'Active (' + sys.prefix + ')' if in_venv else 'Not Active'}")


if __name__ == "__main__":
    main()
