import numpy as np
import matplotlib.pyplot as plt


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
    def __init__(self, dynamics, state_cost, control_cost):
        self.dynamics = dynamics
        self.state_cost = state_cost
        self.control_cost = control_cost

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

            cost = self._cost(xs, us, True)
            if abs(cost - prev_cost) > 1E-3:
                prev_cost = cost
                continue
            print(f"Terminating")
            self._cost(xs, [0 * u for u in us], True)
            break

        return xs, us

    def _cost(self, xs, us, debug=False):
        x_cost = 0
        for x in xs:
            dx = x - self.x_goal
            x_cost += dx.T.dot(self.state_cost).dot(dx)

        u_cost = 0
        for u in us:
            du = u - self.u_goal
            u_cost += du.T.dot(self.control_cost).dot(du)

        if debug:
            print(f"cost() x_cost {x_cost} u_cost {u_cost}")

        return x_cost + u_cost

    def _backward_pass(self, xs, us):
        def lx(x):
            dx = x - self.x_goal
            return dx.dot(2 * self.state_cost)

        def lu(u):
            du = u - self.u_goal
            return du.dot(2 * self.control_cost)

        def lxx(x):
            return 2 * self.state_cost

        p = lx(xs[-1])
        P = lxx(xs[-1])

        count = 0
        rho = 1E-6
        while count < 5:
            Ks = []
            ds = []
            Vs = []
            for step in reversed(range(self.steps)):
                x_t = xs[step]
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
        x0 = xs[0]
        J_best = self._cost(xs, us)
        X, U = xs, us

        for alpha in np.linspace(-1.0, 1.0, 20):
            new_us = np.copy(us)
            new_xs = np.copy(xs)
            for step in range(self.steps):
                new_x = new_xs[step]
                x = xs[step]
                K = Ks[step]
                d = ds[step].reshape(-1)

                du = K.dot((new_x - x)) + alpha * d

                new_us[step] += du
                new_xs[step + 1] = self.dynamics(new_xs[step], new_us[step])

            J = self._cost(new_xs, new_us)
            if J < J_best:
                J_best = J
                X = new_xs
                U = new_us

        return X, U


# state: x, y, vx, vy
# control: fx, fy
state_dim = 4
control_dim = 2

state_cost = np.eye(state_dim)
state_cost[2, 2] = 0.1
state_cost[3, 3] = 0.1
control_cost = 1E-3 * np.eye(control_dim)


def dynamics(x, u):
    dt = 0.1
    mass = 1.0

    a = u / mass
    x = np.copy(x)
    x[0] += dt * x[2] + 0.5 * dt * dt * a[0]
    x[1] += dt * x[3] + 0.5 * dt * dt * a[1]
    x[2] += dt * a[0]
    x[3] += dt * a[1]
    return x


x0 = np.array([0.0, 0.0, 0.0, 0.0])
goal = np.array([10.0, 13.0, 0.0, 0.0])

controller = Controller(dynamics, state_cost, control_cost)
steps = 10
max_iters = 10
xs, us = controller.solve(x0, goal, steps, max_iters)

plt.subplot(2, 1, 1)
plt.scatter(xs[:, 0], xs[:, 1])
plt.scatter(goal[0], goal[1], marker='x', color='red')

plt.subplot(2, 1, 2)
for i in range(control_dim):
    plt.plot(np.arange(len(us)), us[:, i], label=f"u{i}")
plt.legend()

plt.show()
