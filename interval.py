from __future__ import annotations
from abc import ABC, abstractmethod
from dataclasses import dataclass
from typing import Self, Literal
import graphviz
import uuid
import time

type Direction = Literal["left", "right"]


def opposite(direction: Direction) -> Direction:
    if direction == "left":
        return "right"
    elif direction == "right":
        return "left"
    else:
        raise ValueError("unreachable")


@dataclass
class Interval:
    lower: float
    upper: float


def intervals_from_tuples(intervals: list[tuple[float, float]]) -> list[Interval]:
    return [Interval(*i) for i in intervals]


class Solution(ABC):
    @abstractmethod
    def __init__(self, intervals: list[Interval]) -> None: ...

    @abstractmethod
    def add(self, interval: Interval) -> None: ...

    @abstractmethod
    def delete(self, interval: Interval) -> None: ...

    @abstractmethod
    def is_covered(self, point: float) -> bool: ...

    @abstractmethod
    def display(self): ...


def node_count(maybe_node: Node | None):
    if maybe_node is None:
        return 0
    else:
        return maybe_node.child_count + 1


@dataclass
class Node:
    type NewRoot = Self  # best hint i can do in this type system
    left_child: Self | None
    right_child: Self | None
    parent: Self | None
    child_count: int

    def get(self, direction: Direction):
        if direction == "left":
            return self.left_child
        elif direction == "right":
            return self.right_child
        else:
            raise ValueError("unreachable")

    def deduce_direction(self, child) -> Direction:
        if child is self.left_child:
            return "left"
        elif child is self.right_child:
            return "right"
        else:
            raise ValueError("unreachable")

    def set_child_two_way(self, direction: Direction, other: Self | None):
        if direction == "left":
            self.left_child = other
        elif direction == "right":
            self.right_child = other
        else:
            raise ValueError("unreachable")
        if other is not None:
            other.parent = self
        return self

    def update_counts(self):
        self.child_count = node_count(self.left_child) + node_count(self.right_child)

    def update_counts_autobalanced(self) -> NewRoot:
        self.update_counts()
        # propagate the correct count one level up to be ready for rotation
        # wait is this even a requirement?
        # if self.parent is not None:
        # self.parent.update_counts()
        left_child_count = node_count(self.left_child)
        right_child_count = node_count(self.right_child)
        if abs(left_child_count - right_child_count) > 2:
            child_for_promotion = (
                self.left_child
                if left_child_count > right_child_count
                else self.right_child
            )
            assert child_for_promotion is not None
            child_for_promotion.rotate()
        if self.parent is not None:
            return self.parent.update_counts_autobalanced()
        return self

    def set_child_two_way_autobalanced(
        self, direction: Direction, other: Self
    ) -> NewRoot:
        self.set_child_two_way(direction, other)
        return self.update_counts_autobalanced()

    def rotate(self):
        parent_before_rotation = self.parent
        assert parent_before_rotation is not None
        grandpa = parent_before_rotation.parent
        direction = parent_before_rotation.deduce_direction(self)

        child_before_rotation = self.get(opposite(direction))

        # yoink the parent down
        self.set_child_two_way(opposite(direction), parent_before_rotation)
        parent_before_rotation.child_count -= node_count(self)

        # connect the nuked subbranch back to the yoinked parent
        parent_before_rotation.set_child_two_way((direction), child_before_rotation)
        parent_before_rotation.child_count += node_count(child_before_rotation)

        # the new subtree root must be respected by parents, if they exist

        if grandpa is None:
            self.parent = None
        else:
            grandpa.set_child_two_way(
                grandpa.deduce_direction(parent_before_rotation), self
            )


@dataclass
class IntervalTreeNode(Node):
    left_point: float
    right_point: float

    def __repr__(self) -> str:
        return f"({self.left_point}, children={self.child_count})"
        return f"({self.left_point}, {self.right_point})"

    @classmethod
    def lone(cls, left_point: float, right_point: float):
        return cls(
            None,
            None,
            None,
            0,
            left_point,
            right_point,
        )

    def display(self):
        def graph_id(node: IntervalTreeNode | None):
            if node is None:
                return str(uuid.uuid4())
            else:
                return str(id(node))

        g = graphviz.Digraph()
        g.node(graph_id(self), str(self))

        def rec(node: IntervalTreeNode, depth=0, ranks=None):
            if ranks is None:
                ranks = {}

            rcid = graph_id(node.right_child)
            lcid = graph_id(node.left_child)
            nid = graph_id(node)

            g.node(lcid, str(node.left_child))
            g.node(rcid, str(node.right_child))
            g.edge(nid, lcid)
            g.edge(nid, rcid)

            ranks.setdefault(depth + 1, []).extend([lcid, rcid])

            if isinstance(node.left_child, IntervalTreeNode):
                rec(node.left_child, depth + 1, ranks)
            if isinstance(node.right_child, IntervalTreeNode):
                rec(node.right_child, depth + 1, ranks)

            return ranks

        ranks = rec(self)

        for depth_nodes in ranks.values():
            with g.subgraph() as s:  # type: ignore
                s.attr(rank="same")
                for i in range(len(depth_nodes) - 1):
                    # Invisible edges to enforce left-to-right order
                    s.edge(depth_nodes[i], depth_nodes[i + 1], style="invis")

        g.render(view=True, cleanup=True)


class IntervalTreeIGuess(Solution):
    def __init__(self, intervals: list[Interval]) -> None:
        assert len(intervals) > 0
        interval = intervals[0]
        self.root = IntervalTreeNode.lone(
            left_point=interval.lower,
            right_point=interval.upper,
        )
        for interval in intervals[1:]:
            self.add(interval)

    def rotate_left(self): ...

    def rotate_right(self): ...

    def add(self, interval: Interval) -> None:
        node = self.root
        while True:
            if interval.lower < node.left_point:
                direction: Direction = "left"
                child = node.left_child
            elif interval.lower > node.left_point:
                direction: Direction = "right"
                child = node.right_child
            else:  # equal, set the value and go
                return  # its a noop for now as i disregard the upper for testing purposes

            if child is None:
                self.root = node.set_child_two_way_autobalanced(
                    direction,
                    IntervalTreeNode.lone(
                        left_point=interval.lower,
                        right_point=interval.upper,
                    ),
                )
            else:
                node = child

    def delete(self, interval: Interval):
        raise NotImplementedError

    def is_covered(self, point: float) -> bool:
        raise NotImplementedError

    def display(self):
        return self.root.display()

def test(solution: type[Solution]):
    s = solution(intervals_from_tuples([(1, 2), (2, 3), (5, 8), (0, 2), (0, 3)]))

    assert not s.is_covered(4)


if __name__ == "__main__":
    a = IntervalTreeIGuess(
        intervals_from_tuples([(1, 3), (0, 1), (0, 2), (2, 4), (5, 6), (7, 8), (9, 10)])
    )
    print(a.root)
    a.display()
    # print(a.intervals)
    # print(a.binary_search(Interval(7.5, 10)))
