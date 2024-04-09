"use client";

import React, {
  createContext,
  useContext,
  useState,
  useEffect,
  ReactNode,
} from "react";

import {
  getInitRCStatus,
  getInitProgStatus,
  getInitMHV4bus,
  getInitMHV4dev,
  getInitMHV4ch,
  getInitMHV4onoff,
  getInitMHV4pol,
} from "@/lib/transformInitData";

import {
  getSSEProgStatus,
  getSSEVoltageArray,
  getSSECurrentArray,
} from "@/lib/transformSSEData";

type RCType = boolean;
type ProgressType = boolean;
type BusType = number[];
type DevType = number[];
type ChType = number[];
type VoltageType = number[];
type CurrentType = number[];
type IsOnType = boolean[];
type IsPositiveType = boolean[];

interface MHV4ContextType {
  rcType: RCType;
  setRCType: (newValue: RCType) => void;
  progressType: ProgressType;
  busArray: BusType;
  devArray: DevType;
  chArray: ChType;
  voltageArray: VoltageType;
  currentArray: CurrentType;
  isOnArray: IsOnType;
  isPositiveArray: IsPositiveType;
}

const defaultState: MHV4ContextType = {
  rcType: false,
  setRCType: () => {},
  progressType: false,
  busArray: [],
  devArray: [],
  chArray: [],
  voltageArray: [],
  currentArray: [],
  isOnArray: [],
  isPositiveArray: [],
};

const MHV4Context = createContext<MHV4ContextType>(defaultState);

interface MHV4ProviderProps {
  children: ReactNode;
}

export const MHV4DataProvider: React.FC<MHV4ProviderProps> = ({ children }) => {
  const [rcType, setRCType] = useState<RCType>(defaultState.rcType);
  const [progressType, setProgressType] = useState<ProgressType>(
    defaultState.progressType,
  );
  const [busArray, setBusArray] = useState<BusType>(defaultState.busArray);
  const [devArray, setDevArray] = useState<DevType>(defaultState.devArray);
  const [chArray, setChArray] = useState<ChType>(defaultState.chArray);

  const [voltageArray, setVolArray] = useState<VoltageType>(
    defaultState.voltageArray,
  );
  const [currentArray, setCurArray] = useState<CurrentType>(
    defaultState.currentArray,
  );

  const [isOnArray, setIsOnArray] = useState<IsOnType>(defaultState.isOnArray);
  const [isPositiveArray, setIsPositiveArray] = useState<IsPositiveType>(
    defaultState.isPositiveArray,
  );

  useEffect(() => {
    const fetchData = async () => {
      try {
        const response = await fetch(`${process.env.NEXT_PUBLIC_INIT_ROUTE}`);
        if (!response.ok) {
          throw new Error("failed to fetch the MHV4 data");
        }
        const initialData = await response.json();
        console.log(initialData);
        const data = JSON.parse(initialData);
        // set initial state
        setRCType(getInitRCStatus(data));
        setProgressType(getInitProgStatus(data));
        setBusArray(getInitMHV4bus(data));
        setDevArray(getInitMHV4dev(data));
        setChArray(getInitMHV4ch(data));
        setIsOnArray(getInitMHV4onoff(data));
        setIsPositiveArray(getInitMHV4pol(data));
      } catch (error) {
        console.error("Failed to fetch initial data:", error);
      }
    };

    fetchData();

    const eventSource = new EventSource(`${process.env.NEXT_PUBLIC_SSE_ROUTE}`);
    eventSource.onopen = (event) => {
      console.log("SSE connection opened: ", event);
    };
    eventSource.onmessage = (event) => {
      console.log("SSE message received: ", event);
      const ssedata = JSON.parse(event.data);
      // set SSE data
      setProgressType(getSSEProgStatus(ssedata));
      setVolArray(getSSEVoltageArray(ssedata));
      setCurArray(getSSECurrentArray(ssedata));
    };
    eventSource.onerror = (event) => {
      console.error("SSE connection error: ", event);
      setVolArray((currentArray) => {
        const newArray = currentArray.map(() => -100000);
        return newArray;
      });
      setCurArray((currentArray) => {
        const newArray = currentArray.map(() => -100000);
        return newArray;
      });
    };

    return () => {
      eventSource.close();
    };
  }, []);

  return (
    <MHV4Context.Provider
      value={{
        rcType,
        setRCType,
        progressType,
        busArray,
        devArray,
        chArray,
        voltageArray,
        currentArray,
        isOnArray,
        isPositiveArray,
      }}
    >
      {children}
    </MHV4Context.Provider>
  );
};

export const useMHV4Data = (): MHV4ContextType => useContext(MHV4Context);
