"use client";

import React from "react";
import { useMHV4Data } from "@/contexts/MHV4Context";

import { Switch } from "@/components/ui/switch";
import { Input } from "@/components/ui/input";
import {
  Table,
  TableBody,
  TableCaption,
  TableCell,
  TableFooter,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";

const processPolArray = (inputArray: boolean[]): string[] => {
  return inputArray.map((value) => {
    if (value) {
      return "+";
    } else {
      return "-";
    }
  });
};

const processOnOffArray = (inputArray: boolean[]): string[] => {
  return inputArray.map((value) => {
    if (value) {
      return "ON";
    } else {
      return "OFF";
    }
  });
};


const processVoltageArray = (inputArray: number[]): string[] => {
  return inputArray.map((value) => {
    if (value < -99999) {
      return "read error!";
    } else {
      return (value * 0.1).toFixed(1).toString();
    }
  });
};

const processCurrentArray = (inputArray: number[]): string[] => {
  return inputArray.map((value) => {
    if (value < -99999) {
      return "read error!";
    } else {
      return (value * 0.001).toFixed(3).toString();
    }
  });
};


const MHV4Table: React.FC = () => {
  const { progressType, busArray, devArray, chArray, voltageArray, currentArray, isOnArray, isPositiveArray } = useMHV4Data();
  const pols = processPolArray(isPositiveArray);
  const voltages = processVoltageArray(voltageArray);
  const currents = processCurrentArray(currentArray);

  let border_color = "border-green-500";
  if (!progressType) {
    border_color = "border-yellow-500";
  }

  return (
    <Table className="border-solid text-center">
      <TableHeader className="bg-blue-100 font-bold">
        <TableRow>
          <TableHead>bus</TableHead>
          <TableHead>dev</TableHead>
          <TableHead>ch</TableHead>
          <TableHead>status</TableHead>
          <TableHead>on/off</TableHead>
          <TableHead>pol</TableHead>
          <TableHead>input (V)</TableHead>
          <TableHead>voltage (V)</TableHead>
          <TableHead>current (uA)</TableHead>
          <TableHead>name</TableHead>
          <TableHead>discription</TableHead>
        </TableRow>
      </TableHeader>
      <TableBody>
        {busArray.map((bus, index) => (
          <TableRow key={bus}>
            <TableCell>{bus}</TableCell>
            <TableCell>{devArray[index]}</TableCell>
            <TableCell>{chArray[index]}</TableCell>
            <TableCell>{isOnArray[index]}</TableCell>
            <TableCell><Switch/></TableCell>
            <TableCell>{pols[index]}</TableCell>
            <TableCell><Input/></TableCell>
            <TableCell>{voltages[index]}</TableCell>
            <TableCell>{currents[index]}</TableCell>
            <TableCell>test</TableCell>
            <TableCell>test</TableCell>
          </TableRow>
        ))}
        <TableRow>
        </TableRow>
      </TableBody>

    </Table>
  )
}

export default MHV4Table;
