interface ApiData {
  data: {
    values: number[];
  }[];
}

export const transformToNumberArrayData = (apiData: ApiData): number[][] => {
  return apiData.data.map((item) => item.values);
};
