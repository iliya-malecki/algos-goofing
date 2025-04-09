from __future__ import annotations
from abc import ABC, abstractmethod
import itertools
from typing import Iterable, Literal, cast, Self, Protocol
from collections import deque
import graphviz


class Solution(ABC):
    @abstractmethod
    def __init__(self, items: list[int]) -> None: ...
    @abstractmethod
    def setitem(self, at: int, value: int) -> None: ...
    @abstractmethod
    def range_sum(self, bottom: int, top: int) -> int: ...


class Boring(Solution):
    def __init__(self, items: list[int]) -> None:
        self.items = items

    def setitem(self, at: int, value: int) -> None:
        self.items[at] = value

    def range_sum(self, bottom: int, top: int) -> int:
        return sum(self.items[bottom : top + 1])


class Cum(Solution):
    def __init__(self, items: list[int]) -> None:
        self.items = items
        self.update_cumsum()

    def update_cumsum(self):
        self.fcum = self.cumsum(self.items)

    def setitem(self, at: int, value: int) -> None:
        self.items[at] = value
        self.update_cumsum()

    @staticmethod
    def cumsum(items: list[int]):
        new = []
        acc = 0
        for item in items:
            acc += item
            new.append(acc)
        return new

    def range_sum(self, bottom: int, top: int) -> int:
        res = self.fcum[top]
        if bottom > 0:
            res -= self.fcum[bottom - 1]
        return res


Hops = Iterable[Literal["0", "1"]]


class Addable(Protocol):
    def __add__(self, other, /) -> Self: ...
    def __radd__(self, other, /) -> Self: ...


class Totalable[T](Protocol):
    def get_total(self) -> T: ...


class Leaf[T: Addable]:
    __slots__ = ("value",)

    def __init__(self, value: T) -> None:
        self.value = value

    def get_total(self):
        return self.value


class Node[T: Addable]:
    def __init__(
        self, left: Self | Leaf[T] | None, right: Self | Leaf[T] | None
    ) -> None:
        self.left = left
        self.right = right
        self.set_sum()

    def get_total(self):
        return self.sum

    def __repr__(self) -> str:
        return f"Node(sum={self.sum})"

    def set_sum(self):
        if isinstance(self.left, Node):
            left_sum = self.left.sum
        elif self.left is None:
            left_sum = 0
        else:
            left_sum = self.left.value

        if isinstance(self.right, Node):
            right_sum = self.right.sum
        elif self.right is None:
            right_sum = 0
        else:
            right_sum = self.right.value

        self.sum = left_sum + right_sum

    def iter_path(self, hops: Hops):
        current = self
        yield current
        for hop in hops:
            if not isinstance(current, Node):
                raise ValueError("a Node can't be a leaf while hops are hopping")
            if hop == "0":
                current = current.left
            elif hop == "1":
                current = current.right
            else:
                raise ValueError("unreachable")
            yield current

    def traverse(self, hops: Hops):
        element = self
        for element in self.iter_path(hops):
            ...
        return element


class Tree[T: Addable](Solution):
    def __init__(self, items: list[int]) -> None:
        self.len = len(items)
        nodes = [Leaf(item) for item in items]
        for degree in itertools.count(1):
            nodes = self.build_level(nodes)
            if len(nodes) == 1:
                self.root = nodes[0]
                self.degree = degree
                return

    def __len__(self):
        return self.len

    @staticmethod
    def build_level(nodes: list[Node] | list[Leaf[int]]):
        results: list[Node] = []
        for i in range(0, len(nodes), 2):
            new_node = Node(
                nodes[i],
                None if i + 1 >= len(nodes) else nodes[i + 1],
            )
            results.append(new_node)
        return results

    def get_hops(self, index: int):
        return cast(
            Hops,
            format(index, "b").rjust(self.degree, "0")[-self.degree :],
        )

    def setitem(self, at: int, value: int) -> None:
        assert at < len(self)
        hops = self.get_hops(at)
        visited = deque(self.root.iter_path(hops))
        leaf = visited.pop()
        assert isinstance(leaf, Leaf)
        leaf.value = value

        while visited:
            branch = visited.pop()
            assert isinstance(branch, Node)
            branch.set_sum()

    def extract_common_hops(
        self, these: list[Literal["0", "1"]], those: list[Literal["0", "1"]]
    ):
        common = []
        for n, (this, that) in enumerate(zip(these, those)):
            if this == that:
                common.append(this)
            else:
                break
        return common, these[n:], those[n:]  # type: ignore

    def range_sum(self, bottom: int, top: int) -> int:
        bottom_hops = list(self.get_hops(bottom))
        top_hops = list(self.get_hops(top))
        common_hops, bottom_hops, top_hops = self.extract_common_hops(
            bottom_hops, top_hops
        )
        start = self.root.traverse(common_hops)
        assert isinstance(start, Node)

        return self.sum_half(bottom_hops, start, "right") + self.sum_half(
            top_hops, start, "left"
        )

    def sum_half(
        self,
        hops: Hops,
        current: Node,
        direction: Literal["left", "right"],
    ):
        visited = deque(current.iter_path(hops))
        last = visited.pop()
        assert isinstance(last, Leaf)
        total = last.value
        visited.popleft()  # we dont need the root since its other half will be computed symmetrically
        while visited:
            parent = visited.pop()
            if (
                getattr(parent, direction) is not last
                and getattr(parent, direction) is not None
            ):
                total += getattr(parent, direction).get_total()
            last = parent
        return total

    def viz(self):
        g = graphviz.Digraph()
        g.node(str(id(self.root)), str(self.root.sum))

        def rec(node: Node):
            g.node(str(id(node.right)), str(node.right and node.right.get_total()))
            g.node(str(id(node.left)), str(node.left and node.left.get_total()))
            g.edge(str(id(node)), str(id(node.left)))
            g.edge(str(id(node)), str(id(node.right)))
            if isinstance(node.left, Node):
                rec(node.left)
            if isinstance(node.right, Node):
                rec(node.right)

        rec(self.root)
        g.render(view=True)


def test(Solution: type[Solution]):
    solution = Solution([1, 2, 3, 4, 5, 6, 7, 8, 9])
    res = solution.range_sum(0, 1)
    assert res == 3, res
    res = solution.range_sum(7, 8)
    assert res == 17, res
    solution.setitem(3, 7)
    res = solution.range_sum(2, 5)
    assert res == 21, res


def test2(Solution: type[Solution]):
    solution = Solution([1, 2, 3, 4, 5, 6, 7, 8, 9])
    res = solution.range_sum(0, 2)
    assert res == 6, res


if __name__ == "__main__":
    test(Cum)
    test2(Tree)
    test(Tree)
