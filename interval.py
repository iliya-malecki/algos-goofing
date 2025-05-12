from __future__ import annotations
from abc import ABC, abstractmethod
from dataclasses import dataclass
from typing import Self, Literal
import graphviz
import uuid

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
        return f"({self.left_point}, {self.right_point})[ch={self.child_count}]"

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

    def overlaps(self, other: Self):
        return other.right_point - self.left_point < (
            other.right_point - other.left_point
        ) + (self.right_point - self.left_point)

    def merge_children(self, direction: Direction):
        while True:
            child = self.get(direction)
            if child is None:
                return
            elif self.overlaps(child):
                # discard the other child since thats covered by self's new interval
                # here, the "-" represents the newly inserted interval
                #       ---------______     <self
                #   _____                   <self.left_child
                #          ____             <self.left_child.right_child
                if direction == "left":
                    self.left_point = child.left_point
                    self.left_child = child.left_child
                elif direction == "right":
                    self.right_point = child.right_point
                    self.right_child = child.right_child
            else:
                # got to a child i should not perform surgery on
                return

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

    def add(self, interval: Interval) -> None:
        node = self.root
        while True:
            # traverse
            if interval.upper < node.left_point and node.left_child is not None:
                node = node.left_child
            elif interval.lower > node.right_point and node.right_child is not None:
                node = node.right_child
            # disjoint; create new children
            elif interval.upper < node.left_point and node.left_child is None:
                self.root = node.set_child_two_way_autobalanced(
                    "left",
                    IntervalTreeNode.lone(
                        left_point=interval.lower,
                        right_point=interval.upper,
                    ),
                )
                return
            elif interval.lower > node.right_point and node.right_child is None:
                self.root = node.set_child_two_way_autobalanced(
                    "right",
                    IntervalTreeNode.lone(
                        left_point=interval.lower,
                        right_point=interval.upper,
                    ),
                )
                return
            # overlap
            else:
                if interval.lower < node.left_point:
                    node.left_point = interval.lower
                    node.merge_children("left")
                if interval.upper > node.right_point:
                    node.right_point = interval.upper
                    node.merge_children("right")
                return

    def delete(self, interval: Interval):
        node = self.root
        while True:
            # traverse
            if interval.upper < node.left_point:
                node = node.left_child
            elif interval.lower > node.right_point:
                node = node.right_child
            # overlap
            elif interval.upper < node.right_point and interval.lower > node.left_point:
                # the most annoying case of splitting in the middle.
                node.left_point = interval.upper
                self.root = node.set_child_two_way_autobalanced(
                    "left",
                    IntervalTreeNode(
                        node.left_child,
                        None,
                        node,
                        node_count(node.left_child) + 1,
                        node.left_point,
                        interval.lower,
                    ),
                )
                return
            elif interval.upper < node.right_point:
                node.left_point = interval.upper
                node = node.left_child
            elif interval.lower > node.left_point:
                node.right_point = interval.lower
                node = node.right_child
            if node is None:
                return

    def is_covered(self, point: float) -> bool:
        node = self.root
        while True:
            if point < node.left_point:
                node = node.left_child
            elif point > node.right_point:
                node = node.right_child
            else:
                return True
            if node is None:
                return False

    def display(self):
        return self.root.display()


def test(solution: type[Solution]):
    s = solution(intervals_from_tuples([(1, 2), (2.1, 3), (5, 8), (0, 2), (0, 3)]))
    assert not s.is_covered(4)
    assert s.is_covered(2.5)

    s.delete(Interval(2.7, 6.3))
    s.display()
    assert s.is_covered(2.05)
    assert s.is_covered(2.1)
    assert not s.is_covered(2.8)
    assert not s.is_covered(6.2)


if __name__ == "__main__":
    test(IntervalTreeIGuess)
