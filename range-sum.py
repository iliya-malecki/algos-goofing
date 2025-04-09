from __future__ import annotations
from abc import ABC, abstractmethod
import itertools
from typing import Iterable, Literal, cast, Self
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


class Node:
    def __init__(self, left: Self | None, right: Self | None, sum: int) -> None:
        self.left = left
        self.right = right
        self.sum = sum

    def __repr__(self) -> str:
        return f"Node(sum={self.sum})"

    def is_leaf(self):
        return self.left is None and self.right is None

    def set_sum(self):
        left_sum = self.left.sum if self.left is not None else 0
        right_sum = self.right.sum if self.right is not None else 0
        self.sum = left_sum + right_sum

    @classmethod
    def leaf(cls, sum: int):
        return cls(None, None, sum)

    @classmethod
    def branch(cls, left: Self | None, right: Self | None):
        node = cls(left, right, 0)
        node.set_sum()
        return node

    def iter_path(self, hops: Hops):
        current = self
        yield current
        for hop in hops:
            if hop == "0":
                current = current.left
            elif hop == "1":
                current = current.right
            else:
                raise ValueError("unreachable")
            if current is None:
                raise ValueError("a Node can't be None while hops are hopping")
            yield current

    def traverse(self, hops: Hops):
        element = self
        for element in self.iter_path(hops):
            ...
        return element


class Tree(Solution):
    def __init__(self, items: list[int]) -> None:
        self.len = len(items)
        nodes = [Node.leaf(item) for item in items]
        for degree in itertools.count(1):
            nodes = self.build_level(nodes)
            if len(nodes) == 1:
                self.root = nodes[0]
                self.degree = degree
                return

    def __len__(self):
        return self.len

    @staticmethod
    def build_level(nodes: list[Node]):
        results: list[Node] = []
        for i in range(0, len(nodes), 2):
            new_node = Node.branch(
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
        assert leaf.is_leaf()
        leaf.sum = value

        while visited:
            visited.pop().set_sum()

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
        assert last.is_leaf()
        total = last.sum
        visited.popleft()  # we dont need the root since its other half will be computed symmetrically
        while visited:
            parent = visited.pop()
            if (
                getattr(parent, direction) is not last
                and getattr(parent, direction) is not None
            ):
                total += getattr(parent, direction).sum
            last = parent
        return total

    def viz(self):
        g = graphviz.Digraph()
        g.node(str(id(self.root)), str(self.root.sum))

        def rec(node: Node):
            g.node(str(id(node.right)), str(node.right and node.right.sum))
            g.node(str(id(node.left)), str(node.left and node.left.sum))
            g.edge(str(id(node)), str(id(node.left)))
            g.edge(str(id(node)), str(id(node.right)))
            if node.left is not None:
                rec(node.left)
            if node.right is not None:
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
