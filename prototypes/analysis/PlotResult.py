from dataclasses import dataclass

from matplotlib import pyplot as plt


@dataclass
class PlotResult:
    fig: plt.Figure
    ax: plt.Axes

    def show(self) -> None:
        plt.show()

    def save(self, path: str, dpi: int = 200, transparent: bool = False) -> None:
        self.fig.savefig(path, dpi=dpi, bbox_inches="tight", transparent=transparent)