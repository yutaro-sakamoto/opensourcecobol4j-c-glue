struct small_data {
  char char_data;
  short short_data;
  int int_data;
};

void init(int x, int* y)
{
   printf("C function init x=%d, y=%d\n", x, *y);
}

int destroy(struct small_data *data, int i)
{
    return i;
}
