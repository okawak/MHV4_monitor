"use client";

import React from "react";
import { useMHV4Data } from "@/contexts/MHV4Context";

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

const MHV4Table: React.FC = () => {
  const { rcType, progressType, busArray, devArray, chArray, voltageArray, currentArray, isOnArray, isPositiveArray } = useMHV4Data();
  return (
    <Table>
      <TableHeader>
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
            <TableCell>{isOnArray[index]}</TableCell>
            <TableCell>{isPositiveArray[index]}</TableCell>
            <TableCell>{voltageArray[index]}</TableCell>
            <TableCell>{voltageArray[index]}</TableCell>
            <TableCell>{currentArray[index]}</TableCell>
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
