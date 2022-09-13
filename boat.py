import numpy as np
import matplotlib as mpl
import matplotlib.pyplot as plt


def min_angle_diff(lhs, rhs):
    diff = lhs - rhs
    while diff > np.pi:
        diff -= np.pi
    while diff <= -np.pi:
        diff += np.pi
    return diff


class Boat(object):
    def __init__(self):
        self.length = 11.0
        self.width = 4.0

        self.mass = 10.0
        self.moment = 10.0

        # inertial
        self.pos = np.zeros(2)
        self.heading = 0.0
        self.heading_rate = 0.0

        # boat relative
        self.v = np.zeros(2)
        self.thrusters = np.array([
            [-0.4 * self.length, -0.4 * self.width, 0.0],
            [-0.4 * self.length, 0.4 * self.width, 0.0],
        ])

        # forces
        self.u = np.zeros(self.thrusters.shape[0])

        # hydrodynamic coeffs
        self.cx, self.cy, self.cr = 0.9, 0.75, 0.8


    def rot(self):
        c, s = np.cos(self.heading), np.sin(self.heading)
        return np.array([[c, -s], [s, c]])

    def inertial_velocity(self):
        return self.rot().dot(self.v)

    def draw(self):
        R = self.rot()
        thruster_pos = self.pos + self.thrusters[:, :2]
        for ((x, y, theta), u) in zip(self.thrusters, self.u):
            p = R.dot(np.array([x, y])) + self.pos
            plt.arrow(p[0], p[1],
                      u * np.cos(theta + self.heading),
                      u * np.sin(theta + self.heading), length_includes_head=True, color='black')

        v = self.inertial_velocity()
        plt.arrow(self.pos[0], self.pos[1], v[0], v[1], length_includes_head=True, color='red')
        plt.scatter(self.pos[0], self.pos[1], c='red', s=1)

        l, w = 0.5 * self.length, 0.5 * self.width
        pts = np.array([[l, w], [l, -w], [-l, -w], [-l, w], [l, w],])
        pts = R.dot(pts.T).T
        pts += self.pos
        plt.plot(pts[:, 0], pts[:, 1], c='red')

    def update(self, dt):
        print(f"x:{self.pos} v:{self.v}, h:{self.heading:.3f}, w:{self.heading_rate:.3f}")
        self.pos += self.inertial_velocity() * dt
        self.heading += self.heading_rate * dt

        for ((x, y, theta), u) in zip(self.thrusters, self.u):
            f = u * np.array([np.cos(theta), np.sin(theta)])

            # f = m * dv/dt
            self.v += dt * f / self.mass

            # torque = r * f * sin(alpha)
            r = np.sqrt(x ** 2 + y ** 2)
            alpha = min_angle_diff(np.arctan(y / x), theta)
            self.heading_rate += dt * r * u * np.sin(alpha) / self.moment

        self.v[0] *= self.cx
        self.v[1] *= self.cy
        self.heading_rate *= self.cr

b = Boat()
b.pos = np.array([10.0, 20.0])
b.heading = 0.25 * np.pi
b.v = np.array([5.0, 0.0])
b.u = np.array([1.0, 1.0])

for i in range(20):
    b.update(1.0)
    b.draw()

    b.u[1] += 0.1
    if i > 10:
        b.u = np.zeros_like(b.u)

plt.xlim(0, 100)
plt.ylim(0, 100)
plt.gca().set_aspect('equal', 'box')

plt.show()

