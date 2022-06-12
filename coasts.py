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
CHECKPOINT_PATH = "/tmp/coasts/checkpoints"

h = random.getrandbits(128)


def export_dataset(out_dir, images):
    with open('/Users/mlangford/Downloads/GSHHS_f_NAmerica.txt') as f:
        data = f.readlines()

    points = np.array([list(map(float, d.replace(' ', '').replace('\n', '').split('\t'))) for d in data])

    def to_xy(ref, lons, lats):
        m_per_deg_lat = 111132.954 - 559.822 * np.cos(2.0 * np.radians(ref[1])) + 1.175 * np.cos(4.0 * np.radians(ref[1]))
        m_per_deg_lon = 111132.954 * np.cos(np.radians(ref[1]))

        return np.vstack(((lons - ref[0]) * m_per_deg_lon, (lats - ref[1]) * m_per_deg_lat)).T

    fig = plt.figure(figsize=(2,2))
    for i in range(images):
        plt.cla()
        ref = points[np.random.randint(len(points))]
        xy = to_xy(ref, points[:, 0], points[:, 1])
        plt.fill(xy[:, 0], xy[:, 1], 'green', linewidth=0.0)
        plt.ylim(-50000, 50000)
        plt.xlim(-50000, 50000)
        plt.axis('off')
        path = f"{out_dir}/{h:02x}_{i:05d}.png"
        print("Saving:", path)

        fig.savefig(path, facecolor='blue', bbox_inches='tight', pad_inches=0)


DIM = 128

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
        #min_dim = min(image.shape[0], image.shape[1])
        output.append(image[:DIM, :DIM, 1])

    assert len(output) >= count
    return output

x_train = np.stack(load_or_generate(TRAIN_PATH, 5000))
x_test = np.stack(load_or_generate(TEST_PATH, 300))

x_train = x_train.astype('float32') / 255.
x_test = x_test.astype('float32') / 255.


class Autoencoder(Model):
  def __init__(self, latent_dim):
    super(Autoencoder, self).__init__()
    self.latent_dim = latent_dim

    self.encoder = tf.keras.Sequential([
        #layers.Conv2D(32, (3, 3), activation='relu', input_shape=(DIM, DIM, 1)),
        #layers.MaxPooling2D(2, strides=2),
        layers.Flatten(),
        layers.Dense(latent_dim, activation='relu'),
    ])
    self.decoder = tf.keras.Sequential([
        layers.Dense(DIM * DIM, activation='sigmoid'),
        layers.Reshape((DIM, DIM, 1)),
    ])


  def call(self, x):
    encoded = self.encoder(x)
    decoded = self.decoder(encoded)
    return decoded


if not os.path.exists(CHECKPOINT_PATH):
    os.mkdir(CHECKPOINT_PATH)
checkpoint_path = f"{CHECKPOINT_PATH}/cp.ckpt"
checkpoint_dir = os.path.dirname(checkpoint_path)
checkpoint_callback = tf.keras.callbacks.ModelCheckpoint(filepath=checkpoint_path, save_weights_only=True, verbose=1)

autoencoder = Autoencoder(256)
autoencoder.compile(optimizer='adam', loss=losses.MeanSquaredError())
autoencoder.build((None, DIM, DIM, 1))
autoencoder.summary()
autoencoder.fit(x_train, x_train,
                epochs=1000,
                shuffle=True,
                validation_data=(x_test, x_test),
                callbacks=[checkpoint_callback])

encoded_imgs = autoencoder.encoder(x_test).numpy()
decoded_imgs = autoencoder.decoder(encoded_imgs).numpy()


n = 10
plt.figure(figsize=(20, 4))
for i in range(n):
  # display original
  ax = plt.subplot(2, n, i + 1)
  plt.imshow(x_test[i])
  plt.title("original")
  plt.gray()
  ax.get_xaxis().set_visible(False)
  ax.get_yaxis().set_visible(False)

  # display reconstruction
  ax = plt.subplot(2, n, i + 1 + n)
  plt.imshow(decoded_imgs[i])
  plt.title("reconstructed")
  plt.gray()
  ax.get_xaxis().set_visible(False)
  ax.get_yaxis().set_visible(False)

plt.show()
