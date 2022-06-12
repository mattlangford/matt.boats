import numpy as np
import matplotlib.pyplot as plt
import os
import random

import pandas as pd
import tensorflow as tf

from sklearn.metrics import accuracy_score, precision_score, recall_score
from sklearn.model_selection import train_test_split
from tensorflow.keras import layers, losses
from tensorflow.keras.models import Model


TRAIN_PATH = "/tmp/coasts/train"
TEST_PATH = "/tmp/coasts/test"

h = random.getrandbits(128)


def export_dataset(out_dir, images):
    with open('/Users/mlangford/Downloads/GSHHS_f_NAmerica.txt') as f:
        data = f.readlines()

    points = np.array([list(map(float, d.replace(' ', '').replace('\n', '').split('\t'))) for d in data])

    def to_xy(ref, lons, lats):
        m_per_deg_lat = 111132.954 - 559.822 * np.cos(2.0 * np.radians(ref[1])) + 1.175 * np.cos(4.0 * np.radians(ref[1]))
        m_per_deg_lon = 111132.954 * np.cos(np.radians(ref[1]))

        return np.vstack(((lons - ref[0]) * m_per_deg_lon, (lats - ref[1]) * m_per_deg_lat)).T

    for i in range(images):
        plt.cla()
        fig = plt.figure(figsize=(6,6))
        ref = points[np.random.randint(len(points))]
        xy = to_xy(ref, points[:, 0], points[:, 1])
        plt.fill(xy[:, 0], xy[:, 1], 'green', linewidth=0.0)
        plt.ylim(-50000, 50000)
        plt.xlim(-50000, 50000)
        plt.axis('off')
        path = f"{out_dir}/{h:02x}_{i:05d}.png"
        print("Saving:", path)

        fig.savefig(path, facecolor='blue', bbox_inches='tight', pad_inches=0)

def load_or_generate(path, count):
    if not os.path.exists(path):
        os.mkdir(path)

    images = [p for p in os.listdir(path) if p.endswith('.png')]
    if len(images) < count:
        to_generate = count - len(images)
        print(f"Needed {count} images but only got {len(images)}. Generating the {to_generate} now")
        export_dataset(path, to_generate)

    output = []
    for image in [plt.imread(f"{path}/{p}") for p in os.listdir(path) if p.endswith('.png')]:
        min_dim = min(image.shape[0], image.shape[1])
        output.append(image[:min_dim, :min_dim, 1])

    assert len(output) >= count
    return output

train = load_or_generate(TRAIN_PATH, 5)
test = load_or_generate(TEST_PATH, 1)

print(len(train), train[0].shape)
