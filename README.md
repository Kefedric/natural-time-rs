# natural-time-rs
A Rust command-line implementation of **Natural Time**.

## About Natural Time

This project is a Rust implementation of the Natural Time concept originally created by [Sylvain441](https://github.com/sylvain441).

- Original Project: [https://github.com/sylvain441/natural-time](https://github.com/sylvain441/natural-time)

## Features
- Accurate astronomical year start (December Solstice)
- 13 moons of 28 days + Rainbow days
- 360° natural solar time (SUN°)
- Sunrise and Sunset in Natural Time degrees
- Day of the Week (1–7)
- Integer and decimal precision support

## Usage
```
nt-rs <latitude> <longitude> [precision]
```

# Example input
```
nt-rs 22.9231 12.2896
```

```
nt-rs 73.0 -20.91 1
```
# Example output
```
014)06)28 100°53 NT-3 DOW3 ↑062° ↓280°
```

# Output format
In the following order:
```
014)06)28 = DATE 
100°53    = TIME
NT-3      = TIME DISTANCE FROM GREENWICH
DOW3      = DAY OF WEEK (3 in this exemple) 
↑062°     = SUNRISE TIME
↓280°     = SUNSET TIME
```

## License
This project is licensed under the **GNU General Public License v3.0** (GPLv3).  
You are free to use, modify, and distribute it **as long as your derivatives remain open source** under the same license.

## Credits & Acknowledgment
Concept & Original Implementation: [Sylvain441](https://github.com/sylvain441).

Developed by a Rust beginner with heavy assistance from Grok (xAI).
