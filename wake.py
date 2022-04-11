import numpy as np
import matplotlib.pyplot as plt


class Grid(object):
    def __init__(self, dim, resolution):
        self.dim = dim
        self.grid = np.zeros((dim, dim))
        self.resolution = resolution  # m/cell

    def show(self, disp=True):
        flat = self.grid.flatten()
        plt.imshow(self.grid, vmin=-1.0, vmax=1.0, cmap='Blues')
        if disp:
            plt.show()

    def xy_to_grid(self, xy):
        scaled = np.array(xy) / self.resolution
        center = np.array([self.dim, self.dim]) / 2
        col, row = (center + scaled).T
        return np.array([row, col])

    def grid_to_xy(self, row_col):
        center = np.array([self.dim, self.dim]) / 2
        row, col = row_col
        return (np.array([row, col]).T - center) * self.resolution

    def _out_of_bounds(self, row_col):
        row, col = row_col
        return np.any(col < 0) or np.any(col >= self.dim) or np.any(row < 0) or np.any(row >= self.dim)

    def set(self, row_col, value):
        if self._out_of_bounds(row_col):
            return

        row, col = row_col.astype(np.int)
        self.grid[row, col] = value

    def set_xy(self, xy, value):
        self.set(self.xy_to_grid(xy), value)

    def get(self, row_col):
        if self._out_of_bounds(row_col):
            return None

        row, col = row_col.astype(np.int)
        return self.grid[row, col]

    def get_xy(self, xy):
        return self.get(self.xy_to_grid(xy))

    def clear(self):
        self.grid = np.zeros_like(self.grid)

    def coordinates(self):
        arr = np.arange(0, self.dim, 1)
        r, c = np.meshgrid(arr, arr)
        return np.array([r, c]).reshape(2, -1).T

    def coordinates_xy(self):
        return self.grid_to_xy(self.coordinates().T)


class Wavefront(object):
    def __init__(self, start, t0, max_speed):
        self.start = np.array(start)
        self.t0 = t0

        self.max_speed = max_speed
        self.wave_speeds = np.linspace(0.0, self.max_speed, 10)[1:]
        self.wavelengths = np.array(
            [self.wave_speed_to_length(s) for s in self.wave_speeds])

    def at(self, t, xy):
        xy = np.array(xy).reshape(-1, 2)
        total = np.zeros(len(xy))
        dt0 = t - self.t0
        if dt0 <= 0.0:
            return total

        dist = np.linalg.norm(xy - self.start, axis=1)
        for speed, length in zip(self.wave_speeds, self.wavelengths):
            x_wave = 0.5 * speed * dt0
            x_point = dist

            freq = speed / length
            delta = x_wave - x_point
            x = np.maximum(delta, np.zeros_like(delta))
            phase = 2 * np.pi * freq * x / speed

            k1 = 1.0
            k2 = 0.1
            end_x = 0.1
            end = np.maximum(x - end_x, np.zeros_like(x))
            scale = 0.9 * length / self.wavelengths[-1]
            total += scale * np.exp(-k1 * end) * \
                np.exp(-k2 * dist) * np.sin(phase)

        return total

    def wave_length_to_speed(self, wavelength):
        G = 9.81
        return np.sqrt((G * wavelength) / (2 * np.pi))

    def wave_speed_to_length(self, wave_speed):
        G = 9.81
        return wave_speed * wave_speed * 2 * np.pi / G


class Path(object):
    def __init__(self, start, end, speed):
        self.start = np.array(start)
        self.speed = speed
        self.direction = (end - self.start) / np.linalg.norm(self.start - end)

    def at(self, t):
        return self.offset(self.start, t * self.speed)

    def closest(self, xy):
        perp = self.perp()
        return (self.start - xy).dot(perp) * perp + xy

    def when(self, xy):
        diff = (xy - self.start)
        l = np.linalg.norm(diff)
        if l > 0.1 and abs((diff / l).dot(self.direction)) < 0.99:
            print(
                f"xy: {xy}, start: {self.start}, error: {diff.dot(self.direction)}")
            raise Exception()

        return l / self.speed

    def offset(self, xy, l):
        return xy + self.direction * l

    def perp(self):
        return np.array([-self.direction[1], self.direction[0]])


grid = Grid(256, 0.1)
start = grid.grid_to_xy([0, 0])
end = grid.grid_to_xy([grid.dim - 1, grid.dim - 1])
v = 1.0
p = Path(start, end, v)
xys = grid.coordinates_xy()

ts = np.arange(0.0, 30.0, 0.5)
fronts = []
for t_now in ts[::2]:
    xy = p.at(t_now)
    fronts.append(Wavefront(xy, t_now, v))

for i, t_now in enumerate(ts):
    grid.clear()

    total = np.zeros_like(grid.grid)
    for front in fronts:
        total += front.at(t_now, xys).reshape(grid.grid.shape)

    grid.set_xy(xys, total.flatten())
    grid.show(False)

    xy = p.at(t_now)
    r, c = grid.xy_to_grid(xy)
    plt.scatter(c, r)

    start_xy = p.at(t_now - 10)
    perp = p.perp()
    dist = np.linalg.norm(start_xy - xy)
    r0, c0 = grid.xy_to_grid(start_xy + perp * dist *
                             np.tan(np.radians(19.47)))
    r1, c1 = grid.xy_to_grid(start_xy - perp * dist *
                             np.tan(np.radians(19.47)))
    plt.plot([c0, c, c1], [r0, r, r1], c='g')
    r0, c0 = grid.xy_to_grid(start_xy + perp * dist * np.tan(np.radians(45.0)))
    r1, c1 = grid.xy_to_grid(start_xy - perp * dist * np.tan(np.radians(45.0)))
    plt.plot([c0, c, c1], [r0, r, r1], c='r')

    print(f"Saving frame {i} at {t_now:.3f}s")
    plt.savefig(f'/tmp/wake/{i:03d}.png')
    plt.clf()
