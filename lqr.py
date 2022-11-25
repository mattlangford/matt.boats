import numpy as np
import matplotlib.pyplot as plt
import scipy.optimize


def partials(f, x0, eps=1E-4):
    d = []
    for i in range(len(x0)):
        xp = np.zeros_like(x0)
        xn = np.zeros_like(x0)
        xp[i] += eps
        xn[i] -= eps
        d.append((f(xp) - f(xn)) / (2.0 * eps))

    return np.array(d).T


class Controller(object):
    def __init__(self, dynamics, state_cost, goal_cost, control_cost, dx):
        self.dynamics = dynamics
        self.state_cost = state_cost
        self.goal_cost = goal_cost
        self.control_cost = control_cost
        self.dx = dx

        self.state_dim = self.state_cost.shape[0]
        self.control_dim = self.control_cost.shape[0]

    def solve(self, x0, goal, steps=10, max_iters=10):
        assert len(x0) == len(goal)

        self.x_goal = goal
        self.u_goal = np.zeros(self.control_dim)
        self.steps = steps

        us = []
        xs = [x0]

        for step in range(self.steps):
            zero = np.zeros(self.control_dim)
            us.append(zero)
            xs.append(self.dynamics(xs[-1], zero))

        prev_cost = 0
        for it in range(max_iters):
            print(f"Iter {it}")
            Ks, ds, Vs = self._backward_pass(xs, us)
            xs, us = self._forward_pass(xs, us, Ks, ds, Vs)

            x_cost, u_cost, x_goal_cost = self._cost(xs, us, True)
            cost = x_cost + u_cost + x_goal_cost
            print(f"x: {x_cost:.3f} u: {u_cost:.3f} goal: {x_goal_cost:.3f}")
            if abs(cost - prev_cost) > 1E-3:
                prev_cost = cost
                continue
            print(f"Terminating")
            break

        return xs, us

    def _cost(self, xs, us, debug=False):
        x_cost = 0
        for x in xs[:-1]:
            dx = self.dx(x, self.x_goal)
            x_cost += dx.T.dot(self.state_cost).dot(dx)

        dx = self.dx(xs[-1], self.x_goal)
        x_goal_cost = dx.T.dot(self.goal_cost).dot(dx)

        u_cost = 0
        for u in us:
            du = u - self.u_goal
            u_cost += du.T.dot(self.control_cost).dot(du)

        if debug:
            return x_cost, u_cost, x_goal_cost

        return x_cost + u_cost + x_goal_cost

    def _backward_pass(self, xs, us):
        def lx(x, cost=self.state_cost):
            dx = self.dx(x, self.x_goal)
            return dx.T.dot(2 * cost)

        def lu(u):
            du = u - self.u_goal
            return du.T.dot(2 * self.control_cost)

        def lxx(x, cost=self.state_cost):
            return 2 * cost

        p = lx(xs[-1], self.goal_cost)
        P = lxx(xs[-1], self.goal_cost)

        count = 0
        rho = 1.0
        while count < 5:
            count += 1

            Ks = []
            ds = []
            Vs = []
            for step in reversed(range(self.steps)):
                x_t = xs[step - 1]
                u_t = us[step]

                lxx = self.state_cost + self.state_cost.T
                luu = self.control_cost + self.control_cost.T
                A = partials(lambda x: self.dynamics(x, u_t), x_t)
                B = partials(lambda u: self.dynamics(x_t, u), u_t)

                Qx = lx(x_t) + A.T.dot(p)  # (state, 1)
                Qu = lu(u_t) + B.T.dot(p)  # (control, 1)
                Qxx = lxx + A.T.dot(P).dot(A)  # (state, state)
                Quu = luu + B.T.dot(P).dot(B)  # (control, control)
                Qux = B.T.dot(P).dot(A)  # (control, state)
                Qxu = Qux.T

                try:
                    _inv = -np.linalg.inv(Quu + rho * np.eye(Quu.shape[0]))
                except Exception as e:
                    print("Inv failed:", e)
                    rho *= 2.0
                    continue

                K = _inv.dot(Qux)
                d = _inv.dot(Qu)

                P = Qxx + K.T.dot(Quu).dot(K) + K.T.dot(Qux) + Qxu.dot(K)
                p = Qx + K.T.dot(Quu).dot(d) + K.T.dot(Qu) + Qxu.dot(d)

                V = d.T.dot(Qu) + 0.5 * d.T.dot(Quu).dot(d)

                Ks.append(K)
                ds.append(d)
                Vs.append(V)

            return Ks, ds, Vs
        raise

    def _forward_pass(self, xs, us, Ks, ds, Vs):
        print(Ks[0])
        def run(alpha):
            new_us = np.copy(us)
            new_xs = np.copy(xs)
            for step in range(self.steps):
                new_x = new_xs[step]
                x = xs[step]
                K = Ks[step]
                d = ds[step].reshape(-1)

                dx = self.dx(new_x, x)
                du = K.dot(dx) + alpha * d

                new_us[step] += du
                new_xs[step + 1] = self.dynamics(new_xs[step], new_us[step])

            J = self._cost(new_xs, new_us)
            return J, new_xs, new_us

        solution = scipy.optimize.minimize_scalar(lambda a: run(a)[0])
        _, X, U = run(solution.x)
        return np.array(X), np.array(U)

np.set_printoptions(edgeitems=30, linewidth=100000, 
    formatter=dict(float=lambda x: "%.3g" % x))

state_names = ["x", "y", "vx", "vy", "heading", "heading rate"]
control_names = ["fwd_force", "torque"]
state_dim = len(state_names)
control_dim = len(control_names)

def dx(lhs, rhs):
    diff = lhs - rhs
    diff[4] = (diff[4] + np.pi) % (2 * np.pi) - np.pi
    return diff

def dynamics(x, u):
    dt = 0.1
    mass = 10.0
    inertia = 1.0

    fwd_a = u[0] / mass
    heading_a = u[1] / inertia

    world_a_x = fwd_a * np.cos(x[4])
    world_a_y = fwd_a * np.sin(x[4])

    decay = 0.8

    x = np.copy(x)
    x[0] += dt * x[2] + 0.5 * dt * dt * world_a_x
    x[1] += dt * x[3] + 0.5 * dt * dt * world_a_y
    x[2] += decay * dt * world_a_x
    x[3] += decay * dt * world_a_y
    x[4] += dt * x[5] + 0.5 * dt * dt * heading_a
    x[5] += decay * dt * heading_a

    x[4] = (x[4] + np.pi) % (2 * np.pi) - np.pi
    return x

state_cost = np.zeros((state_dim, state_dim))
goal_cost = np.zeros((state_dim, state_dim))
control_cost = np.zeros((control_dim, control_dim))

state_cost[0, 0] = 1E1
state_cost[1, 1] = 1E1
state_cost[4, 4] = 1E-1
state_cost[5, 5] = 1E0
goal_cost[0, 0] = 1E1
goal_cost[1, 1] = 1E1
goal_cost[2, 2] = 1E-2
goal_cost[3, 3] = 1E-2
goal_cost[4, 4] = 1E1
goal_cost[5, 5] = 1E1
control_cost[0, 0] = 1E-1
control_cost[1, 1] = 1E-2

x0 = np.zeros(state_dim)
x0[4] = 0.0
x0[2] = 10.0
x0[3] = 0.0
goal = np.zeros(state_dim)
goal[0] = 2.0
goal[1] = 0.0
goal[4] = 0.0

controller = Controller(dynamics, state_cost, goal_cost, control_cost, dx)
steps = 10
max_iters = 100
xs, us = controller.solve(x0, goal, steps, max_iters)
us *= 10

plt.figure(figsize=(5, 5))
plt.quiver(xs[:, 0], xs[:, 1], np.cos(xs[:, 4]), np.sin(xs[:, 4]), np.arange(len(xs)))
plt.quiver(goal[0], goal[1], np.cos(goal[4]), np.sin(goal[4]), color='red')

plt.figure(figsize=(5, 2 * control_dim + 6))
for i in range(control_dim):
    plt.subplot(control_dim + 3, 1, i + 1)
    plt.title(f"u{i}: {control_names[i]}")
    plt.plot(np.arange(len(us)), us[:, i])

def x_cost(x):
    d = dx(x, goal)
    return d.T.dot(state_cost).dot(d)
def u_cost(u):
    return u.T.dot(control_cost).dot(u)
def cost_goal(x):
    d = dx(x, goal)
    return d.T.dot(goal_cost).dot(d)

x_costs = list(map(x_cost, xs[:-1]))
u_costs = list(map(u_cost, us))
costs = [cost_goal(xs[-1])]
for x, u in zip(xs[:-1:], us[::-1]):
    costs = [costs[-1] + x_cost(x) + u_cost(u)] + costs
print(f"Final cost: {sum(costs)}")
print(sum(costs))

plt.subplot(control_dim + 3, 1, control_dim + 1)
plt.title("X Cost")
plt.plot(np.arange(len(x_costs)), x_costs, label="x")
plt.subplot(control_dim + 3, 1, control_dim + 2)
plt.title("U Cost")
plt.plot(np.arange(len(u_costs)), u_costs, label="u")
plt.subplot(control_dim + 3, 1, control_dim + 3)
plt.title("Total Cost")
plt.plot(np.arange(len(costs)), costs, label="total")
plt.tight_layout()

plt.figure(figsize=(5, 2 * state_dim))
for i in range(state_dim):
    plt.subplot(1 + state_dim, 1, i + 1)
    plt.title(f"x{i}: {state_names[i]}")
    plt.plot(np.arange(len(xs)), xs[:, i])

plt.tight_layout()
plt.show()
