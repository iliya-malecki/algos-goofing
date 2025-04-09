from __future__ import annotations
from abc import ABC, abstractmethod
import itertools
from typing import Iterable, Literal, cast
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


class Node:
    def __init__(self, left: Node | None, right: Node | None, sum: int) -> None:
        self.left = left
        self.right = right
        self.sum = sum

    def __repr__(self) -> str:
        return f"Node(sum={self.sum})"

    def set_sum(self):
        left_sum = self.left.sum if self.left is not None else 0
        right_sum = self.right.sum if self.right is not None else 0
        self.sum = left_sum + right_sum

    @classmethod
    def leaf(cls, sum: int):
        return cls(None, None, sum)

    @classmethod
    def branch(cls, left: Node | None, right: Node | None):
        node = Node(left, right, 0)
        node.set_sum()
        return node


class Tree(Solution):
    def __init__(self, items: list[int]) -> None:
        nodes = [Node.leaf(item) for item in items]
        for degree in itertools.count(1):
            nodes = self.build_level(nodes)
            if len(nodes) == 1:
                self.root = nodes[0]
                self.degree = degree
                return

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
            Iterable[Literal["0", "1"]],
            format(index, "b").rjust(self.degree, "0")[-self.degree :],
        )

    def setitem(self, at: int, value: int) -> None:
        # padded and truncated to the degree of the tree to start at the bottom
        hops = self.get_hops(at)
        current = self.root
        visited = deque[Node]()
        for hop in hops:
            visited.append(current)
            if hop == "0":
                if current.left is None:
                    current.left = Node.leaf(0)
                current = current.left
            elif hop == "1":
                if current.right is None:
                    current.right = Node.leaf(0)
                current = current.right
            else:
                raise ValueError("unreachable")
        current.sum = value  # set the leaf
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
        current = self.root
        for hop in common_hops:
            if hop == "0":
                assert current.left is not None
                current = current.left
            elif hop == "1":
                assert current.right is not None
                current = current.right
            else:
                raise ValueError("unreachable")
        return self.sum_half(bottom_hops, current, "right") + self.sum_half(
            top_hops, current, "left"
        )

    def sum_half(
        self,
        hops: Iterable[Literal["0", "1"]],
        current: Node,
        direction: Literal["left", "right"],
    ):
        visited = deque[Node]()
        for hop in hops:
            visited.append(current)
            if hop == "0":
                assert current.left is not None
                current = current.left
            elif hop == "1":
                assert current.right is not None
                current = current.right
            else:
                raise ValueError("unreachable")
        total = current.sum
        visited.popleft()  # we dont need the root
        while visited:
            parent = visited.pop()
            if (
                getattr(parent, direction) is not current
                and getattr(parent, direction) is not None
            ):
                total += getattr(parent, direction).sum
            current = parent
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
