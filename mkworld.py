worldsize_x = 100
worldsize_y = 50

with open("world.save", "w") as world:
  for height in range(worldsize_y):
    for _ in range(worldsize_x):
      if height < worldsize_y/2:
        world.write("0\n")
      else:
        world.write("1\n")