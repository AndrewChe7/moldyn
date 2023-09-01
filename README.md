# Molecular dynamic toolchain

> **Warning**
> This project is *Work in progress*, so now there is not much of the features.
> 
> Implemented:
> 
> * Serialization/Deserialization of current state and sequence of states (frames) for visualization.
> * Simple methods such as Verlet integration, Berendsen thermostat/barostat, Lennard Jones potential. 
> * CLI application for steps calculation, macro params calculation, state initialization
> * GUI application with visualization, animation replay, graphs for macro parameters.
>
> TODO:
>
> * Plugin system with [extism](https://github.com/extism/extism) for custom integrators, potentials, barostates and thermostates.
> * Structures save/load and other tools
> * GPU computation for some steps of calculation

## Usage (CLI)

Firstly you need to initialize first state, it could be done with:
```bash
./moldyn-cli -f initialization_file.json initialize -t uniform -s 10 10 10 -n Argon -m 66.335 -r 0.071 -l 3.338339 -T 273.15
```

Now you can only initialize uniform grid (like in gases). `-s` is the size of the grid (in example 10x10x10). `-n`, `-m`, `-r` are for name, mass and radius of particle.
`-l` sets length of lattice cell edge. `-T` sets starting temperature for particle velocities initialization.

Next you can run calculations:

```bash
./moldyn-cli -f ./initialization_file.json -b 20000 solve -o steps.json -c 100000 -i verlet-method --thermostat berendsen --thermostat-params 10 -T 300 --barostat berendsen --barostat-params 1 5 -P 1.01325 -t 0.002 
```

You could get more information about parameters from `-h` on each command. `-b` is for backup frequency. It saves all frames to different files. In future there should be parameter for "frames per nanosecond" or something like iterations per frame.

After that command you will get multiple files like `steps.20000.json`, number means last saved frame in file. You can already watch the animation in gui application or calculate macro parameters:

```bash
./moldyn-cli -f ./steps.20000.json solve-macro-parameters -o ./steps.json -A
```

This command calculates all macro parameters. If you want to calculate only some of them you can specify it with parameters, check `-h` for more information.

## Usage (GUI)

Just launch application and you will see the interface. It works with wgpu, so it mostly cross-platform (I hope). You can open files you made with CLI and watch the animation.

