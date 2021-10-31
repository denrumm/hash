# automatically generated by the FlatBuffers compiler, do not modify

# namespace: 

import flatbuffers
from flatbuffers.compat import import_numpy
np = import_numpy()

class StateSnapshotSync(object):
    __slots__ = ['_tab']

    @classmethod
    def GetRootAs(cls, buf, offset=0):
        n = flatbuffers.encode.Get(flatbuffers.packer.uoffset, buf, offset)
        x = StateSnapshotSync()
        x.Init(buf, n + offset)
        return x

    @classmethod
    def GetRootAsStateSnapshotSync(cls, buf, offset=0):
        """This method is deprecated. Please switch to GetRootAs."""
        return cls.GetRootAs(buf, offset)
    # StateSnapshotSync
    def Init(self, buf, pos):
        self._tab = flatbuffers.table.Table(buf, pos)

    # StateSnapshotSync
    def AgentPool(self, j):
        o = flatbuffers.number_types.UOffsetTFlags.py_type(self._tab.Offset(4))
        if o != 0:
            x = self._tab.Vector(o)
            x += flatbuffers.number_types.UOffsetTFlags.py_type(j) * 4
            x = self._tab.Indirect(x)
            from Batch import Batch
            obj = Batch()
            obj.Init(self._tab.Bytes, x)
            return obj
        return None

    # StateSnapshotSync
    def AgentPoolLength(self):
        o = flatbuffers.number_types.UOffsetTFlags.py_type(self._tab.Offset(4))
        if o != 0:
            return self._tab.VectorLen(o)
        return 0

    # StateSnapshotSync
    def AgentPoolIsNone(self):
        o = flatbuffers.number_types.UOffsetTFlags.py_type(self._tab.Offset(4))
        return o == 0

    # StateSnapshotSync
    def MessagePool(self, j):
        o = flatbuffers.number_types.UOffsetTFlags.py_type(self._tab.Offset(6))
        if o != 0:
            x = self._tab.Vector(o)
            x += flatbuffers.number_types.UOffsetTFlags.py_type(j) * 4
            x = self._tab.Indirect(x)
            from Batch import Batch
            obj = Batch()
            obj.Init(self._tab.Bytes, x)
            return obj
        return None

    # StateSnapshotSync
    def MessagePoolLength(self):
        o = flatbuffers.number_types.UOffsetTFlags.py_type(self._tab.Offset(6))
        if o != 0:
            return self._tab.VectorLen(o)
        return 0

    # StateSnapshotSync
    def MessagePoolIsNone(self):
        o = flatbuffers.number_types.UOffsetTFlags.py_type(self._tab.Offset(6))
        return o == 0

    # StateSnapshotSync
    def CurrentStep(self):
        o = flatbuffers.number_types.UOffsetTFlags.py_type(self._tab.Offset(8))
        if o != 0:
            return self._tab.Get(flatbuffers.number_types.Int64Flags, o + self._tab.Pos)
        return 0

def Start(builder): builder.StartObject(3)
def StateSnapshotSyncStart(builder):
    """This method is deprecated. Please switch to Start."""
    return Start(builder)
def AddAgentPool(builder, agentPool): builder.PrependUOffsetTRelativeSlot(0, flatbuffers.number_types.UOffsetTFlags.py_type(agentPool), 0)
def StateSnapshotSyncAddAgentPool(builder, agentPool):
    """This method is deprecated. Please switch to AddAgentPool."""
    return AddAgentPool(builder, agentPool)
def StartAgentPoolVector(builder, numElems): return builder.StartVector(4, numElems, 4)
def StateSnapshotSyncStartAgentPoolVector(builder, numElems):
    """This method is deprecated. Please switch to Start."""
    return StartAgentPoolVector(builder, numElems)
def AddMessagePool(builder, messagePool): builder.PrependUOffsetTRelativeSlot(1, flatbuffers.number_types.UOffsetTFlags.py_type(messagePool), 0)
def StateSnapshotSyncAddMessagePool(builder, messagePool):
    """This method is deprecated. Please switch to AddMessagePool."""
    return AddMessagePool(builder, messagePool)
def StartMessagePoolVector(builder, numElems): return builder.StartVector(4, numElems, 4)
def StateSnapshotSyncStartMessagePoolVector(builder, numElems):
    """This method is deprecated. Please switch to Start."""
    return StartMessagePoolVector(builder, numElems)
def AddCurrentStep(builder, currentStep): builder.PrependInt64Slot(2, currentStep, 0)
def StateSnapshotSyncAddCurrentStep(builder, currentStep):
    """This method is deprecated. Please switch to AddCurrentStep."""
    return AddCurrentStep(builder, currentStep)
def End(builder): return builder.EndObject()
def StateSnapshotSyncEnd(builder):
    """This method is deprecated. Please switch to End."""
    return End(builder)