interface MHV4Data {
  idc: number;
  bus: number;
  dev: number;
  ch: number;
  current: number;
  is_on: boolean;
  is_positive: boolean;
}

interface MHV4Response {
  mhv4_data_array: MHV4Data[];
  is_rc: boolean;
  is_progress: boolean;
}

export const getInitRCStatus = (mhv4Response: MHV4Response): boolean =>
  mhv4Response.is_rc;
export const getInitProgStatus = (mhv4Response: MHV4Response): boolean =>
  mhv4Response.is_progress;

function getInitMHV4Data(
  mhv4Response: MHV4Response,
  key: keyof MHV4Data,
): (number | boolean)[][] {
  return mhv4Response.mhv4_data_array.reduce(
    (acc: (number | boolean)[][], mod, index) => {
      if ((index + 1) % 4 !== 0) {
        if (acc.length === 0 || acc[acc.length - 1].length === 4) {
          acc.push([mod[key]]);
        } else {
          acc[acc.length - 1].push(mod[key]);
        }
      }
      return acc;
    },
    [],
  );
}

export const getInitMHV4bus = (mhv4Response: MHV4Response): number[][] =>
  getInitMHV4Data(mhv4Response, "bus") as number[][];

export const getInitMHV4dev = (mhv4Response: MHV4Response): number[][] =>
  getInitMHV4Data(mhv4Response, "dev") as number[][];

export const getInitMHV4ch = (mhv4Response: MHV4Response): number[][] =>
  getInitMHV4Data(mhv4Response, "ch") as number[][];

export const getInitMHV4voltage = (mhv4Response: MHV4Response): number[][] =>
  getInitMHV4Data(mhv4Response, "current") as number[][];

export const getInitMHV4onoff = (mhv4Response: MHV4Response): boolean[][] =>
  getInitMHV4Data(mhv4Response, "is_on") as boolean[][];

export const getInitMHV4pol = (mhv4Response: MHV4Response): boolean[][] =>
  getInitMHV4Data(mhv4Response, "is_positive") as boolean[][];
