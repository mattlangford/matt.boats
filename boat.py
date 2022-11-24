import numpy as np
import matplotlib as mpl
import matplotlib.pyplot as plt


def rotation_matrix(theta):
    c, s = np.cos(theta), np.sin(theta)
    return np.array([[c, -s], [s, c]])


def angle_clamp(angle, min_angle=-np.pi, max_angle=np.pi):
    while angle > np.pi:
        angle -= np.pi
    while angle <= -np.pi:
        angle += np.pi
    return angle


def min_angle_diff(lhs, rhs):
    return angle_clamp(lhs - rhs)


def plot_rect(pos, heading, v, length, width, color='red'):
    v = rotation_matrix(heading).dot(v)
    plt.arrow(pos[0], pos[1], v[0], v[1], length_includes_head=True, color=color)
    plt.scatter(pos[0], pos[1], c=color, s=1)

    l, w = 0.5 * length, 0.5 * width
    pts = np.array([[l, w], [l, -w], [-l, -w], [-l, w], [l, w],])
    pts = rotation_matrix(heading).dot(pts.T).T
    pts += pos
    plt.plot(pts[:, 0], pts[:, 1], c=color, linewidth=0.1)


# params: [length, width, mass, moment, cx, cy, cr, thruster0_pos, ..., thrusterN_pos]
# x: [X, Y, x_dot, y_dot, heading, heading_dot]
# u: [f0, f1, ... fN]

class Params(object):
    def __init__(self):
        self.length = 11.0
        self.width = 4.0
        self.mass = 10.0
        self.moment = 10.0

        self.thrusters = np.array([
            [-0.4 * self.length, -0.4 * self.width, 0.0],
            [-0.4 * self.length, 0.4 * self.width, 0.0],
        ])

        # hydrodynamic coeffs
        self.cx, self.cy, self.cr = 0.9, 0.75, 0.8


class State(object):
    def __init__(self):
        self.pos = np.zeros(2)
        self.v = np.zeros(2)
        self.heading = 0.0
        self.heading_rate = 0.0

    def as_array(self):
        return np.array([self.pos[0], self.pos[1], self.v[0], self.v[1], self.heading, self.heading_rate])

    @staticmethod
    def from_array(cls, array):
        s = State()
        s.pos[0] = array[0]
        s.pos[1] = array[1]
        s.v[0] = array[2]
        s.v[1] = array[3]
        s.heading = array[4]
        s.heading_rate = array[5]
        return s

    def angle_indicies(self):
        return [4]

    def dim(self):
        return self.as_array().shape[0]

    def difference(self, rhs):
        diff = self.as_array() - rhs.as_array()
        for angle in self.angle_indicies():
            diff[angle] = angle_clamp(diff[angle])
        return diff

    def __repr__(self):
        x = np.round(self.pos, 3)
        v = np.round(self.v, 3)
        h = np.degrees(self.heading)
        w = np.degrees(self.heading_rate)
        return f"pos: {x} heading: {h:.3f} v: {v} heading_rate: {w:.3f}"


class Control(object):
    def __init__(self, dof):
        self.u = np.zeros(dof)

    def as_array(self):
        return self.u

    @staticmethod
    def from_array(self, arr):
        c = Control(len(arr))
        c.u = arr
        return c

    def dim(self):
        return self.u.shape[0]


def dynamics(state, control, params, dt):
    new = State()

    new.pos = state.pos + rotation_matrix(state.heading).dot(state.v) * dt
    new.heading = state.heading + state.heading_rate * dt

    new.v = np.copy(state.v)
    new.heading_rate = state.heading_rate

    for ((x, y, theta), u) in zip(params.thrusters, control.u):
        f = u * np.array([np.cos(theta), np.sin(theta)])

        # f = m * dv/dt
        new.v += dt * f / params.mass

        # torque = r * f * sin(alpha)
        r = np.sqrt(x ** 2 + y ** 2)
        alpha = min_angle_diff(np.arctan(y / x), theta)
        new.heading_rate += dt * r * u * np.sin(alpha) / params.moment

    new.v[0] *= params.cx
    new.v[1] *= params.cy
    new.heading_rate *= params.cr
    return new




def loss(state, control, params, state_cost, control_cost):
    s = state.as_array()
    u = control.as_array()

    cost = 0
    cost += s.T.dot(state_cost).dot(s)
    cost += u.T.dot(control_cost).dot(u)
    return cost


def goal_loss(state, params, goal, goal_cost):
    state_diff = state.difference(goal)
    return state_diff.T.dot(goal_cost).dot(state_diff)


class Boat(object):
    def __init__(self):
        self.params = Params()
        self.state = State()
        self.control = Control(self.params.thrusters.shape[0])

    def draw(self):
        R = rotation_matrix(self.state.heading)
        thruster_pos = self.state.pos + self.params.thrusters[:, :2]
        for ((x, y, theta), u) in zip(self.params.thrusters, self.control.u):
            p = R.dot(np.array([x, y])) + self.state.pos
            plt.arrow(p[0], p[1],
                      u * np.cos(theta + self.state.heading),
                      u * np.sin(theta + self.state.heading),
                      length_includes_head=True, color='black')

        plot_rect(self.state.pos, self.state.heading, self.state.v, self.params.length, self.params.width, 'red')

    def update(self, dt):
        self.state = dynamics(self.state, self.control, self.params, dt)


    def loss(self, goal):
        Q = np.zeros((self.state.dim(), self.state.dim()))
        Q[2, 2] = 1.0
        Q[3, 3] = 1.0
        R = np.eye(self.control.dim())
        W = np.eye(self.state.dim())
        return loss(self.state, self.control, self.params, Q, R) + goal_loss(self.state, self.params, goal, W)


def interpolate(start, end, steps):
    pos_diff = end.pos - start.pos
    v_diff = end.v - start.v
    heading_diff = min_angle_diff(end.heading, start.heading)
    heading_rate_diff = end.heading_rate - start.heading_rate

    states = []
    for step in range(steps):
        s = State()
        s.pos = start.pos + step * pos_diff / steps
        s.v = start.v + step * v_diff / steps
        s.heading = angle_clamp(start.heading + step * heading_diff / steps)
        s.heading_rate = start.pos + step * heading_diff / steps
        states.append(s)
    return states



g = State()
g.pos = np.array([50.0, 60.0])
g.heading = 0.25 * np.pi
g.v = np.array([0.0, 0.0])
plt.arrow(g.pos[0], g.pos[1], 5 * np.cos(g.heading), 5 * np.sin(g.heading), width=0.25, length_includes_head=True, color='green')

b = Boat()
b.state.pos = np.array([10.0, 20.0])
b.state.heading = 0.25 * np.pi
b.state.v = np.array([5.0, 0.0])
b.control.u = np.array([5.0, 5.0])

for _ in range(10):
    print(b.loss(g))
    b.update(1.0)
    b.draw()

plt.xlim(0, 100)
plt.ylim(0, 100)
plt.gca().set_aspect('equal', 'box')

#plt.show()

