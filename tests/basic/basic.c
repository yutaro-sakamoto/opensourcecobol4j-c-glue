struct small_data {
  char char_data;
  short short_data;
  int int_data;
};

void init(struct small_data *data, char x, int y)
{
    data->char_data = x;
    data->short_data = x;
    data->int_data = x;
}

int destroy(struct small_data *data, int i)
{
    return i;
}
