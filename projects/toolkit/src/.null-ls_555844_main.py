import typer

app = typer.Typer()


@app.command()
def create(item: str):
    print(f"Creating item: {item}")


def main():
    app()


if __name__ == "__main__":
    main()
