struct small_data {
  char char_data;
  short short_data;
  int int_data;
};

void init(int x)
{
   printf("C function init x=%d\n", x);
}

int destroy(struct small_data *data, int i)
{
    return i;
}
