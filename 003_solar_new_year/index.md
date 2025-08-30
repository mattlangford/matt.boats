---
date: 2025-08-29
---

# Solar New Year
_Why be close, when you can be exact?_

New year countdowns often count down to midnight on January first.
Midnight, of course, is an arbitrary construct - maybe instead we should count down to solar midnight?
The goal then becomes calculating when solar midnight will occur, and to convince all your friends to countdown to that time instead. I will focus on the former and leave the latter to you.

## Solar Midnight
According to [Wikipedia](https://en.wikipedia.org/wiki/Midnight), solar midnight happens halfway between sunset and sunrise, when the sun is closest to [nadir](https://en.wikipedia.org/wiki/Nadir) (down).


Unfortunately, it's hard to observe the position of the sun at night directly. With clear days around the end of December and periodic sextant measurements of the position of the sun around noon, you could come up with a pretty good estimate though: 12 hours after when the sun is highest on the sky.

Thankfully for us, it's also possible to make predictions with python library [astropy](https://www.astropy.org)!

## Astropy
I've played around with astropy a little bit for various projects, but found this was a pretty easy routine to come up with (especially since AI did most of it).
Behind the scenes, astropy pulls astronomical ephemeris data from a few sources like [this one](https://naif.jpl.nasa.gov/pub/naif/generic_kernels/spk/planets/). This contains historical and prediction position information about objects in the solar system.

We can use this to find the exact time when the sun is lowest in the sky during our new years party.
```python
location = astropy.coordinates.EarthLocation(
    lat=..., lon=..., height=0)
```

We'll start with a rough guess about when midnight will occur - normal midnight will do for this. Compute a few samples in this range.
```python
tz = ZoneInfo("America/New_York")
start = datetime(2025, 1, 1, 23, 30, tzinfo=tz)
end = datetime(2026, 1, 1, 0, 30, tzinfo=tz)

SAMPLES = 100
times = astropy.time.Time(
    np.linspace(start.jd, end.jd, SAMPLES), format='jd')
```



Next, compute the position of the sun at each of our reference times in the local [AltAz](https://docs.astropy.org/en/stable/api/astropy.coordinates.AltAz.html) frame.
This converts the 3D position of the sun into an altitude and azimuth on the sky at a given location. The sun will generally be at some negative altitude during the night.
```python
def solar_altitude_at(time):
    time = astropy.time.Time(
        time, format='jd')
    altaz_frame = astropy.coordinates.AltAz(
        obstime=time, location=location)
    sun = astropy.coordinates.get_sun(time)
    sun_altaz = sun.transform_to(altaz_frame)
    return sun_altaz.alt.degree

altitudes = solar_altitude_at(times)
```


Now that we have a list of solar altitudes at different times, we can compute the minimum altitude. This of course won't be the exact minimum (unless we sampled very finely), so compute the lower and upper time bounds where the actual minimum will occur.
```python
min_altitude_index = np.argmin(altitudes)
guess = times[min_altitude_index]
lower_bound = times[min_altitude_index - 1]
upper_bound = times[min_altitude_index + 1]
```


Finally, we can run a search for the actual minimum by giving scipy those bounds and the objective function which returns the altitude of the sun.
```python
result = scipy.optimize.minimize_scalar(
    solar_altitude_at,
    bounds=(lower_bound.jd, upper_bound.jd),
    method="bounded")
solar_midnight = astropy.time.Time(
    result.x, format='jd')
```

## Results
Plotting the array of altitudes and solar midnight looks about right!
![results](plot.png "Results")

With the final result for solar midnight for my location:
```
2026-01-01 00:22:51.594465-05:00
```

See you then!

PS: I think you could come up with at least the latitude of my party from this, but maybe the exact lat/lon. Good luck.