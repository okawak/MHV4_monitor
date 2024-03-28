type SSEType = [voltage: number[], current: number[], is_progress: boolean];

const getSSEArray = (array: number[]): number[][] => {
  return array.reduce((acc: number[][], value, index) => {
    if ((index + 1) % 4 !== 0) {
      if (acc.length === 0 || acc[acc.length - 1].length === 4) {
        acc.push([value]);
      } else {
        acc[acc.length - 1].push(value);
      }
    } else {
      acc[acc.length - 1].push(value);
    }
    return acc;
  }, []);
};

export const getSSEProgStatus = (sseResponse: SSEType): boolean =>
  sseResponse[2];

export const getSSEVoltageArray = (sseResponse: SSEType): number[][] => {
  return getSSEArray(sseResponse[0]);
};

export const getSSECurrentArray = (sseResponse: SSEType): number[][] => {
  return getSSEArray(sseResponse[1]);
};
